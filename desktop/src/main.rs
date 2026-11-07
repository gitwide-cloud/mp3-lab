#![cfg_attr(target_os="windows", windows_subsystem="windows")]
use eframe::egui;
use std::{path::{Path,PathBuf},process::{Command,Child,Stdio},sync::mpsc::{self,Receiver},thread,time::Duration};
mod skin;

#[derive(Default)]
struct Editor {
    input: Option<PathBuf>, duration:f64, start:f64, end:f64, fade_in:f64, fade_out:f64,
    gain:f64, normalize:bool, player:Option<Child>,
    exporting:Option<Receiver<Result<PathBuf,String>>>, status:String,
    waveform:Vec<[f32;2]>, wave_job:Option<Receiver<Result<Vec<[f32;2]>,String>>>, peak_db:Option<f64>,
    preview_job:Option<Receiver<Result<PathBuf,String>>>, preview_path:Option<PathBuf>,
    started:Option<std::time::Instant>, offset:f64, preview_start:f64, preview_end:f64,
    paused:bool, repeat:bool, volume:f32, show_about:bool, show_tools:bool,
    logo:Option<egui::TextureHandle>, screenshot_frames:u32,
}
impl Drop for Editor { fn drop(&mut self) {self.stop();} }
impl Editor {
    fn stop(&mut self){self.kill_player();self.preview_job=None;self.started=None;self.offset=0.0;self.paused=false;if let Some(p)=self.preview_path.take(){let _=std::fs::remove_file(p);}}
    fn kill_player(&mut self){if let Some(mut p)=self.player.take(){let _=p.kill();let _=p.wait();}}
    fn elapsed(&self)->f64{self.offset+self.started.map(|s|s.elapsed().as_secs_f64()).unwrap_or(0.0)}
    fn pause(&mut self){
        if self.player.is_some(){self.offset=self.elapsed();self.started=None;self.kill_player();self.paused=true;self.status="Paused".into();}
        else if self.paused {self.resume();}
    }
    fn resume(&mut self){
        let Some(path)=&self.preview_path else{return};
        match Command::new(tool("ffplay")).args(["-hide_banner","-loglevel","error","-nodisp","-autoexit","-volume"])
            .arg(format!("{:.0}",self.volume)).arg("-ss").arg(format!("{:.5}",self.offset)).arg("-i").arg(path)
            .stdout(Stdio::null()).stderr(Stdio::null()).spawn(){
            Ok(p)=>{self.player=Some(p);self.started=Some(std::time::Instant::now());self.paused=false;self.status="Playing preview".into();},
            Err(e)=>self.status=format!("Playback unavailable: {e}")
        }
    }
    fn seek_preview(&mut self,delta:f64){if self.preview_path.is_some(){let playing=self.player.is_some();self.offset=(self.elapsed()+delta).clamp(0.0,(self.preview_end-self.preview_start-0.01).max(0.0));self.started=None;self.kill_player();if playing{self.resume();}}}
    fn length(&self)->f64{(self.end-self.start).max(0.0)}
    fn clamp(&mut self){
        self.start=self.start.clamp(0.0,self.duration);
        self.end=self.end.clamp(self.start,self.duration);
        self.fade_in=self.fade_in.clamp(0.0,self.length());
        self.fade_out=self.fade_out.clamp(0.0,(self.length()-self.fade_in).max(0.0));
    }
    fn open(&mut self){
        let Some(p)=rfd::FileDialog::new().add_filter("Audio",&["mp3","wav","flac","m4a","aac","ogg","opus"]).pick_file() else{return};
        self.load_path(p);
    }
    fn load_path(&mut self,p:PathBuf){
        self.stop();
        match duration(&p){
            Ok(d) if d>0.0 && d.is_finite()=>{
                self.duration=d;self.start=0.0;self.end=d;self.fade_in=0.0;self.fade_out=0.0;
                self.gain=0.0;self.normalize=false;self.input=Some(p.clone());self.status="Reading waveform…".into();
                self.waveform.clear();self.peak_db=None;
                let(tx,rx)=mpsc::channel();self.wave_job=Some(rx);thread::spawn(move||{let _=tx.send(read_waveform(&p,d));});
            }
            Ok(_)=>self.status="Invalid track duration".into(),
            Err(e)=>self.status=e
        }
    }
    fn filter(&self)->String{
        let mut f=vec!["asetpts=PTS-STARTPTS".to_string()];
        if self.normalize {f.push("loudnorm=I=-16:TP=-3:LRA=11".into())}
        if self.gain.abs()>0.001 {f.push(format!("volume={:.2}dB",self.gain))}
        if self.fade_in>0.0 {f.push(format!("afade=t=in:st=0:d={:.4}",self.fade_in))}
        if self.fade_out>0.0 {f.push(format!("afade=t=out:st={:.4}:d={:.4}",self.length()-self.fade_out,self.fade_out))}
        f.push("alimiter=limit=0.95:attack=5:release=50:level=0".into());
        f.join(",")
    }
    fn play(&mut self,edited:bool){
        self.stop();
        let Some(p)=self.input.clone() else{return};
        if self.length()<=0.0{return}
        let filter=if edited{self.filter()}else{"anull".into()};
        self.preview_start=self.start;self.preview_end=self.end;
        let start=self.start;let length=self.length();let(tx,rx)=mpsc::channel();self.preview_job=Some(rx);
        self.status=if edited{"Preparing edited preview…".into()}else{"Preparing original preview…".into()};
        thread::spawn(move||{
            let output=std::env::temp_dir().join(format!("mp3-lab-preview-{}-{}.wav",std::process::id(),std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()));
            let result=Command::new(tool("ffmpeg")).args(["-hide_banner","-nostdin","-y","-loglevel","error","-ss"]).arg(format!("{start:.5}"))
                .arg("-i").arg(p).arg("-t").arg(format!("{length:.5}")).args(["-vn","-af"]).arg(filter).args(["-c:a","pcm_s16le","-ar","48000"]).arg(&output).output();
            let result=match result{Ok(r) if r.status.success()=>Ok(output.clone()),Ok(r)=>Err(String::from_utf8_lossy(&r.stderr).into_owned()),Err(e)=>Err(e.to_string())};
            let failed=result.is_err();if tx.send(result).is_err()||failed{let _=std::fs::remove_file(output);}
        });
    }
    fn export(&mut self){
        let Some(input)=self.input.clone() else{return};
        if self.length()<=0.0{return}
        let name=format!("{}_clip.mp3",input.file_stem().unwrap_or_default().to_string_lossy());
        let Some(output)=rfd::FileDialog::new().add_filter("MP3",&["mp3"]).set_file_name(&name).save_file() else{return};
        if input==output || (output.exists() && std::fs::canonicalize(&input).ok()==std::fs::canonicalize(&output).ok()) {self.status="Output cannot overwrite source".into();return}
        self.stop();let (tx,rx)=mpsc::channel();self.exporting=Some(rx);
        let start=self.start;let length=self.length();let filters=self.filter();
        self.status="Exporting 256 kbps MP3".into();
        thread::spawn(move||{
            let result=Command::new(tool("ffmpeg")).args(["-hide_banner","-nostdin","-y","-loglevel","error","-ss"])
                .arg(format!("{start:.5}")).arg("-i").arg(&input).arg("-t").arg(format!("{length:.5}"))
                .args(["-vn","-af"]).arg(filters).args(["-codec:a","libmp3lame","-b:a","256k","-id3v2_version","3"])
                .arg(&output).output();
            let outcome=match result{
                Ok(r) if r.status.success()=>Ok(output),
                Ok(r)=>Err(String::from_utf8_lossy(&r.stderr).to_string()),
                Err(e)=>Err(e.to_string())
            };
            let _=tx.send(outcome);
        });
    }
}
fn tool(n:&str)->PathBuf{
    let filename=if cfg!(windows){format!("{n}.exe")}else{n.into()};
    if let Ok(exe)=std::env::current_exe(){
        if let Some(dir)=exe.parent(){let p=dir.join(&filename);if p.exists(){return p}}
    }
    filename.into()
}
fn duration(p:&Path)->Result<f64,String>{
    let o=Command::new(tool("ffprobe")).args(["-v","error","-show_entries","format=duration","-of","default=noprint_wrappers=1:nokey=1"])
        .arg(p).output().map_err(|e|e.to_string())?;
    if !o.status.success(){return Err(String::from_utf8_lossy(&o.stderr).to_string())}
    String::from_utf8_lossy(&o.stdout).trim().parse().map_err(|_|"Duration unavailable".into())
}
fn clock(sec:f64)->String{let m=(sec/60.0).floor() as u64; format!("{}:{:05.2}",m,sec-(m*60) as f64)}
fn read_waveform(path:&Path,duration:f64)->Result<Vec<[f32;2]>,String>{
    use std::io::Read;
    let mut child=Command::new(tool("ffmpeg")).args(["-hide_banner","-loglevel","error","-i"]).arg(path).args(["-vn","-ac","2","-ar","2000","-f","f32le","pipe:1"])
        .stdout(Stdio::piped()).stderr(Stdio::null()).spawn().map_err(|e|e.to_string())?;
    let mut stream=std::io::BufReader::new(child.stdout.take().unwrap());let mut frame=[0u8;8];let mut bins=vec![[0.0f32;2];1600];let mut index=0usize;
    while stream.read_exact(&mut frame).is_ok(){
        let bin=((index as f64/(duration*2000.0))*1600.0) as usize;index+=1;
        if bin<1600{for channel in 0..2{let value=f32::from_le_bytes(frame[channel*4..channel*4+4].try_into().unwrap()).abs();if value.is_finite(){bins[bin][channel]=bins[bin][channel].max(value);}}}
    }
    if !child.wait().map_err(|e|e.to_string())?.success(){return Err("Waveform decoding failed".into())}Ok(bins)
}
impl eframe::App for Editor{
 fn ui(&mut self,ui:&mut egui::Ui,_:&mut eframe::Frame){
    let ctx=ui.ctx().clone();
    if let Some(rx)=&self.exporting{match rx.try_recv(){
        Ok(result)=>{self.status=match result{Ok(p)=>format!("Saved {}",p.display()),Err(e)=>format!("Export failed: {e}")};self.exporting=None;},
        Err(mpsc::TryRecvError::Disconnected)=>{self.status="Export interrupted".into();self.exporting=None;},
        Err(mpsc::TryRecvError::Empty)=>ctx.request_repaint_after(Duration::from_millis(100))
    }}
    if let Some(rx)=&self.wave_job{match rx.try_recv(){
        Ok(result)=>{match result{Ok(w)=>{let peak=w.iter().flat_map(|p|p.iter()).copied().fold(0.0f32,f32::max);self.peak_db=Some(if peak>0.0{20.0*(peak as f64).log10()}else{-120.0});self.waveform=w;self.status="Ready.".into();},Err(e)=>self.status=e}self.wave_job=None;},
        Err(mpsc::TryRecvError::Disconnected)=>{self.wave_job=None;self.status="Waveform interrupted".into();},
        Err(mpsc::TryRecvError::Empty)=>ctx.request_repaint_after(Duration::from_millis(100))
    }}
    if let Some(rx)=&self.preview_job{match rx.try_recv(){
        Ok(result)=>{self.preview_job=None;match result{Ok(p)=>{self.preview_path=Some(p);self.offset=0.0;self.resume();},Err(e)=>self.status=format!("Preview failed: {e}")}},
        Err(mpsc::TryRecvError::Disconnected)=>{self.preview_job=None;self.status="Preview interrupted".into();},
        Err(mpsc::TryRecvError::Empty)=>ctx.request_repaint_after(Duration::from_millis(100))
    }}
    if self.player.is_some(){
        ctx.request_repaint_after(Duration::from_millis(33));
        let done=self.player.as_mut().is_some_and(|p|matches!(p.try_wait(),Ok(Some(_))));
        if done{self.kill_player();self.started=None;self.offset=0.0;if self.repeat{self.resume();}else{self.status="Preview finished".into();}}
    }
    skin::draw(self,ui);
    if let Some(path)=std::env::var_os("MP3_LAB_SCREENSHOT"){
        use eframe::icon_data::IconDataExt;
        if self.wave_job.is_none(){
            self.screenshot_frames+=1;
            if self.screenshot_frames==10{
                let context=ctx.clone();
                ctx.request_screenshot(move|image|{
                    let icon=egui::IconData{width:image.size[0] as u32,height:image.size[1] as u32,rgba:image.pixels.iter().flat_map(|c|c.to_array()).collect()};
                    if let Ok(bytes)=icon.to_png_bytes(){let _=std::fs::write(path,bytes);}context.send_viewport_cmd(egui::ViewportCommand::Close);context.request_repaint();
                });
            }
            ctx.request_repaint_after(Duration::from_millis(60));
        }
    }
 }
}
fn main()->eframe::Result{
    let mut viewport=egui::ViewportBuilder::default().with_inner_size([1040.0,710.0]).with_min_inner_size([930.0,670.0]).with_decorations(false);
    if let Ok(icon)=eframe::icon_data::from_png_bytes(include_bytes!("../../assets/icon.png")){viewport=viewport.with_icon(icon);}
    let opt=eframe::NativeOptions{viewport,..Default::default()};
    eframe::run_native("mp3ϟlab",opt,Box::new(|cc|{
        skin::configure(&cc.egui_ctx);let mut editor=Editor::default();editor.volume=80.0;
        if let Some(path)=std::env::var_os("MP3_LAB_DEMO"){editor.load_path(path.into());editor.start=3.0;editor.end=16.0;editor.fade_in=1.2;editor.fade_out=1.2;editor.normalize=true;editor.clamp();}
        Ok(Box::new(editor))
    }))
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn selection_keeps_fades_inside_clip() {
        let mut e=Editor::default(); e.duration=10.0;e.start=8.0;e.end=20.0;e.fade_in=4.0;e.fade_out=4.0;
        e.clamp();
        assert_eq!((e.start,e.end,e.fade_in,e.fade_out),(8.0,10.0,2.0,0.0));
        e.start=11.0;e.clamp();assert_eq!(e.length(),0.0);
    }
    #[test]
    fn processing_order_and_relative_fade_position() {
        let mut e=Editor::default();e.start=30.0;e.end=35.0;e.normalize=true;e.gain=-6.0;e.fade_in=1.0;e.fade_out=2.0;
        assert_eq!(e.filter(),"asetpts=PTS-STARTPTS,loudnorm=I=-16:TP=-3:LRA=11,volume=-6.00dB,afade=t=in:st=0:d=1.0000,afade=t=out:st=3.0000:d=2.0000,alimiter=limit=0.95:attack=5:release=50:level=0");
    }
    #[test]
    #[ignore = "requires FFmpeg; enabled explicitly by Linux CI"]
    fn ffmpeg_export_smoke() {
        let dir=std::env::temp_dir().join(format!("mp3-lab-test-{}",std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let input=dir.join("source.wav");let output=dir.join("clip.mp3");
        assert!(Command::new(tool("ffmpeg")).args(["-y","-f","lavfi","-i","sine=frequency=440:duration=8"]).arg(&input).output().unwrap().status.success());
        let mut e=Editor::default();e.start=2.0;e.end=6.0;e.normalize=true;e.gain=-3.0;e.fade_in=0.5;e.fade_out=0.5;
        let result=Command::new(tool("ffmpeg")).args(["-y","-ss","2","-i"]).arg(&input).args(["-t","4","-vn","-af"]).arg(e.filter()).args(["-codec:a","libmp3lame","-b:a","256k"]).arg(&output).output().unwrap();
        assert!(result.status.success(),"{}",String::from_utf8_lossy(&result.stderr));
        assert!((duration(&output).unwrap()-4.0).abs()<0.15);
        let probe=Command::new(tool("ffprobe")).args(["-v","error","-select_streams","a:0","-show_entries","stream=codec_name,bit_rate","-of","csv=p=0"]).arg(&output).output().unwrap();
        assert_eq!(String::from_utf8_lossy(&probe.stdout).trim(),"mp3,256000");
        std::fs::remove_dir_all(dir).unwrap();
    }
}

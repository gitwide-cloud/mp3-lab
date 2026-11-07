#![cfg_attr(target_os="windows", windows_subsystem="windows")]
use eframe::egui;
use std::{path::{Path,PathBuf},process::{Command,Child,Stdio},sync::mpsc::{self,Receiver},thread,time::Duration};

#[derive(Default)]
struct Editor {
    input: Option<PathBuf>, duration:f64, start:f64, end:f64, fade_in:f64, fade_out:f64,
    gain:f64, normalize:bool, player:Option<Child>,
    exporting:Option<Receiver<Result<PathBuf,String>>>, status:String,
}
impl Drop for Editor { fn drop(&mut self) {self.stop();} }
impl Editor {
    fn stop(&mut self){if let Some(mut p)=self.player.take(){let _=p.kill();let _=p.wait();}}
    fn length(&self)->f64{(self.end-self.start).max(0.0)}
    fn clamp(&mut self){
        self.start=self.start.clamp(0.0,self.duration);
        self.end=self.end.clamp(self.start,self.duration);
        self.fade_in=self.fade_in.clamp(0.0,self.length());
        self.fade_out=self.fade_out.clamp(0.0,(self.length()-self.fade_in).max(0.0));
    }
    fn open(&mut self){
        let Some(p)=rfd::FileDialog::new().add_filter("Audio",&["mp3","wav","flac","m4a","aac","ogg","opus"]).pick_file() else{return};
        self.stop();
        match duration(&p){
            Ok(d) if d>0.0 && d.is_finite()=>{
                self.duration=d;self.start=0.0;self.end=d;self.fade_in=0.0;self.fade_out=0.0;
                self.gain=0.0;self.normalize=false;self.input=Some(p);self.status="Track loaded".into();
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
        let Some(p)=self.input.as_ref() else{return};
        if self.length()<=0.0{return}
        let filter=if edited{self.filter()}else{"anull".into()};
        match Command::new(tool("ffplay")).args(["-hide_banner","-loglevel","error","-nodisp","-autoexit","-ss"])
            .arg(format!("{:.5}",self.start)).arg("-t").arg(format!("{:.5}",self.length()))
            .arg("-af").arg(filter).arg("-i").arg(p).stdout(Stdio::null()).stderr(Stdio::null()).spawn(){
            Ok(child)=>{self.player=Some(child);self.status="Playing preview".into()},
            Err(e)=>self.status=format!("ffplay unavailable: {e}")
        }
    }
    fn export(&mut self){
        let Some(input)=self.input.clone() else{return};
        if self.length()<=0.0{return}
        let name=format!("{}_clip.mp3",input.file_stem().unwrap_or_default().to_string_lossy());
        let Some(output)=rfd::FileDialog::new().add_filter("MP3",&["mp3"]).set_file_name(&name).save_file() else{return};
        if input==output {self.status="Output cannot overwrite source".into();return}
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
impl eframe::App for Editor{
 fn update(&mut self,ctx:&egui::Context,_:&mut eframe::Frame){
    if let Some(rx)=&self.exporting{
        match rx.try_recv(){
            Ok(Ok(p))=>{self.status=format!("Saved {}",p.display());self.exporting=None},
            Ok(Err(e))=>{self.status=format!("Export failed: {e}");self.exporting=None},
            Err(mpsc::TryRecvError::Disconnected)=>{self.status="Export interrupted".into();self.exporting=None},
            Err(mpsc::TryRecvError::Empty)=>ctx.request_repaint_after(Duration::from_millis(150))
        }
    }
    if let Some(p)=&mut self.player{if let Ok(Some(_))=p.try_wait(){self.player=None}}
    egui::CentralPanel::default().show(ctx,|ui|{
      egui::ScrollArea::vertical().show(ui,|ui|{
        ui.heading(egui::RichText::new("⚡ mp3 lab").color(egui::Color32::LIGHT_GREEN).size(30.0));
        ui.small("CLASSIC PLAYER  •  256 KBPS MP3");
        ui.add_space(10.0);
        egui::Frame::group(ui.style()).fill(egui::Color32::from_rgb(10,25,18)).show(ui,|ui|{
            ui.set_min_width(ui.available_width()-20.0);
            let title=self.input.as_ref().and_then(|p|p.file_name()).map(|s|s.to_string_lossy().into_owned())
                .unwrap_or_else(||"NO TRACK LOADED".into());
            ui.label(egui::RichText::new(title).monospace().color(egui::Color32::LIGHT_GREEN).size(17.0));
            ui.label(egui::RichText::new(format!("IN {}     OUT {}     LENGTH {}",clock(self.start),clock(self.end),clock(self.length())))
               .monospace().color(egui::Color32::LIGHT_GREEN));
        });
        ui.add_space(12.0);
        ui.horizontal(|ui|{
            if ui.button("📂 OPEN AUDIO").clicked(){self.open()}
            if self.input.is_some(){
                if ui.button("▶ EDITED").clicked(){self.play(true)}
                if ui.button("▶ ORIGINAL").clicked(){self.play(false)}
                if ui.button("■ STOP").clicked(){self.stop()}
            }
        });
        if self.input.is_some(){
            ui.separator();ui.heading("01  TRACK SELECTION");
            let (rect,response)=ui.allocate_exact_size(egui::vec2(ui.available_width(),35.0),egui::Sense::click_and_drag());
            ui.painter().rect_filled(rect,2.0,egui::Color32::from_rgb(10,25,18));
            if self.duration>0.0{
                let a=rect.left()+(self.start/self.duration) as f32*rect.width();
                let b=rect.left()+(self.end/self.duration) as f32*rect.width();
                ui.painter().rect_filled(egui::Rect::from_min_max(egui::pos2(a,rect.top()+5.0),egui::pos2(b,rect.bottom()-5.0)),0.0,egui::Color32::DARK_GREEN);
                ui.painter().vline(a,rect.y_range(),egui::Stroke::new(2.0,egui::Color32::LIGHT_GREEN));
                ui.painter().vline(b,rect.y_range(),egui::Stroke::new(2.0,egui::Color32::LIGHT_BLUE));
                if response.clicked()||response.dragged(){
                    if let Some(pos)=response.interact_pointer_pos(){
                        let t=(((pos.x-rect.left())/rect.width()).clamp(0.0,1.0) as f64)*self.duration;
                        if (pos.x-a).abs()<(pos.x-b).abs(){self.start=t.min(self.end)}else{self.end=t.max(self.start)}
                    }
                }
            }
            ui.horizontal(|ui|{
                ui.label("IN");ui.add(egui::DragValue::new(&mut self.start).speed(0.1).range(0.0..=self.duration).suffix(" s"));
                ui.label("OUT");ui.add(egui::DragValue::new(&mut self.end).speed(0.1).range(0.0..=self.duration).suffix(" s"));
            });
            self.clamp();ui.separator();ui.heading("02  FADES");
            ui.horizontal(|ui|{
                ui.label("Fade in");ui.add(egui::DragValue::new(&mut self.fade_in).speed(0.1).range(0.0..=self.length()).suffix(" s"));
                ui.label("Fade out");ui.add(egui::DragValue::new(&mut self.fade_out).speed(0.1).range(0.0..=self.length()).suffix(" s"));
            });
            self.clamp();ui.separator();ui.heading("03  GAIN / LOUDNESS");
            ui.checkbox(&mut self.normalize,"Normalize to -16 LUFS");
            ui.horizontal(|ui|{
                ui.label("Gain");ui.add(egui::Slider::new(&mut self.gain,-24.0..=12.0).suffix(" dB"));
                if ui.button("RESET").clicked(){self.gain=0.0}
            });
            ui.small("Normalize → manual gain → fades → limiter");
            ui.separator();
            if ui.add_enabled(self.exporting.is_none(),egui::Button::new("💾 EXPORT MP3 — 256 KBPS")).clicked(){self.export()}
        }else{ui.label("Open an audio file to begin.")}
        ui.separator();ui.small(&self.status);
      });
    });
 }
}
fn main()->eframe::Result{
    let opt=eframe::NativeOptions{viewport:egui::ViewportBuilder::default().with_inner_size([760.0,650.0]),..Default::default()};
    eframe::run_native("MP3 Lab",opt,Box::new(|cc|{
        let mut v=egui::Visuals::dark();
        v.panel_fill=egui::Color32::from_rgb(28,32,38);
        cc.egui_ctx.set_visuals(v);Ok(Box::<Editor>::default())
    }))
}

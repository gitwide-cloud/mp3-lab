package com.mp3lab.app;

import android.app.Activity;
import android.content.Intent;
import android.graphics.Color;
import android.media.MediaMetadataRetriever;
import android.media.MediaPlayer;
import android.net.Uri;
import android.os.Bundle;
import android.os.Handler;
import android.os.Looper;
import android.provider.OpenableColumns;
import android.database.Cursor;
import android.view.View;
import android.widget.*;
import java.io.*;
import java.util.Locale;
import com.arthenica.ffmpegkit.FFmpegKit;
import com.arthenica.ffmpegkit.ReturnCode;

public class MainActivity extends Activity {
    private static final int PICK=10, SAVE=11;
    private static final int BG=0xff1d2027, PANEL=0xff303640, LED=0xff90f569;
    private final Handler handler=new Handler(Looper.getMainLooper());
    private LinearLayout root;
    private TextView status,filename,clock,gainLabel,fadeInLabel,fadeOutLabel;
    private SeekBar inBar,outBar,gainBar,fadeInBar,fadeOutBar;
    private CheckBox normalize;
    private File input;
    private String name="song";
    private double duration=0,start=0,end=0,fadeIn=0,fadeOut=0,gainDb=0;
    private MediaPlayer player;
    private volatile boolean busy;
    private volatile int previewGeneration;
    private int dp(int x){return (int)(x*getResources().getDisplayMetrics().density);}
    private TextView label(String text,int size,int color) {
        TextView v=new TextView(this);v.setText(text);v.setTextSize(size);v.setTextColor(color);v.setPadding(dp(4),dp(7),dp(4),dp(7));return v;
    }
    private LinearLayout column(){LinearLayout l=new LinearLayout(this);l.setOrientation(LinearLayout.VERTICAL);return l;}
    private void add(LinearLayout l,View v){l.addView(v,new LinearLayout.LayoutParams(-1,-2));}
    private void addSection(String heading,View body){
        LinearLayout block=column();block.setPadding(dp(12),dp(10),dp(12),dp(12));block.setBackgroundColor(PANEL);
        add(block,label(heading,16,0xff7ab6e8));add(block,body);
        LinearLayout.LayoutParams p=new LinearLayout.LayoutParams(-1,-2);p.bottomMargin=dp(10);root.addView(block,p);
    }
    private SeekBar seek(LinearLayout parent,int initial,java.util.function.IntConsumer callback){
        SeekBar b=new SeekBar(this);b.setMax(1000);b.setProgress(initial);
        b.setOnSeekBarChangeListener(new SeekBar.OnSeekBarChangeListener(){
            public void onProgressChanged(SeekBar bar,int progress,boolean fromUser){if(fromUser)callback.accept(progress);}
            public void onStartTrackingTouch(SeekBar bar){}
            public void onStopTrackingTouch(SeekBar bar){}
        });
        add(parent,b);return b;
    }
    private Button button(LinearLayout l,String title,View.OnClickListener onClick){
        Button b=new Button(this);b.setText(title);b.setOnClickListener(onClick);add(l,b);return b;
    }
    @Override public void onCreate(Bundle state){
        super.onCreate(state);
        getWindow().setStatusBarColor(BG);getWindow().setNavigationBarColor(BG);
        ScrollView scroll=new ScrollView(this);scroll.setFillViewport(true);scroll.setBackgroundColor(BG);
        root=column();root.setPadding(dp(12),dp(15),dp(12),dp(25));scroll.addView(root);
        root.setOnApplyWindowInsetsListener((view,insets)->{
            view.setPadding(dp(12)+insets.getSystemWindowInsetLeft(),dp(15)+insets.getSystemWindowInsetTop(),
                dp(12)+insets.getSystemWindowInsetRight(),dp(25)+insets.getSystemWindowInsetBottom());
            return insets;
        });
        add(root,label("mp3⚡lab",30,LED));add(root,label("CLASSIC PLAYER • MP3 256 KBPS",12,0xff7ab6e8));
        button(root,"📂 OPEN AUDIO",v->pick());
        LinearLayout display=column();display.setPadding(dp(12),dp(8),dp(12),dp(8));display.setBackgroundColor(0xff081b10);
        filename=label("NO TRACK LOADED",17,LED);clock=label("00:00 / 00:00",15,LED);
        add(display,filename);add(display,clock);addSection("NOW PLAYING",display);
        LinearLayout selection=column();
        add(selection,label("CLIP START",14,Color.WHITE));
        inBar=seek(selection,0,p->{start=Math.min(end,duration*p/1000);refresh();});
        add(selection,label("CLIP END",14,Color.WHITE));
        outBar=seek(selection,1000,p->{end=Math.max(start,duration*p/1000);refresh();});
        addSection("01  TRACK SELECTION",selection);
        LinearLayout fades=column();
        fadeInLabel=label("Fade in: 0.0 seconds",14,Color.WHITE);add(fades,fadeInLabel);
        fadeInBar=seek(fades,0,p->{fadeIn=30*p/1000.0;refresh();});
        fadeOutLabel=label("Fade out: 0.0 seconds",14,Color.WHITE);add(fades,fadeOutLabel);
        fadeOutBar=seek(fades,0,p->{fadeOut=30*p/1000.0;refresh();});
        addSection("02  FADES",fades);
        LinearLayout gains=column();normalize=new CheckBox(this);
        normalize.setText("Normalize to −16 LUFS");normalize.setTextColor(Color.WHITE);add(gains,normalize);
        gainLabel=label("Gain: +0.0 dB",14,Color.WHITE);add(gains,gainLabel);
        gainBar=seek(gains,667,p->{gainDb=-24+36.0*p/1000;refresh();});
        addSection("03  GAIN / LOUDNESS",gains);
        LinearLayout playerControls=column();
        button(playerControls,"▶ ORIGINAL PREVIEW",v->preview(false));
        button(playerControls,"▶ EDITED PREVIEW",v->preview(true));
        button(playerControls,"■ STOP",v->stop());
        button(playerControls,"💾 EXPORT 256 KBPS MP3",v->save());
        addSection("PLAYBACK & EXPORT",playerControls);
        status=label("Ready. Open an audio file.",13,LED);add(root,status);
        button(root,"OPEN SOURCE LICENSES",v->licenses("licenses"));
        setContentView(scroll);refresh();
    }
    private String time(double s){
        int seconds=(int)Math.max(0,s);
        return String.format(Locale.US,"%d:%02d",seconds/60,seconds%60);
    }
    private void refresh(){
        if(end<start)end=start;
        double len=Math.max(0,end-start);
        fadeIn=Math.min(fadeIn,len);
        fadeOut=Math.min(fadeOut,Math.max(0,len-fadeIn));
        if(fadeInBar!=null)fadeInBar.setProgress((int)(fadeIn/30*1000));
        if(fadeOutBar!=null)fadeOutBar.setProgress((int)(fadeOut/30*1000));
        if(duration>0){inBar.setProgress((int)(start/duration*1000));outBar.setProgress((int)(end/duration*1000));}
        if(clock!=null)clock.setText("IN "+time(start)+"   OUT "+time(end)+"   LENGTH "+time(len));
        if(fadeInLabel!=null)fadeInLabel.setText(String.format(Locale.US,"Fade in: %.1f seconds",fadeIn));
        if(fadeOutLabel!=null)fadeOutLabel.setText(String.format(Locale.US,"Fade out: %.1f seconds",fadeOut));
        if(gainLabel!=null)gainLabel.setText(String.format(Locale.US,"Gain: %+.1f dB",gainDb));
    }
    private void report(String msg){handler.post(()->status.setText(msg));}
    private void licenses(String path){
        try {
            String[] children=getAssets().list(path);
            if(children!=null&&children.length>0){
                new android.app.AlertDialog.Builder(this).setTitle("Open source licenses")
                    .setItems(children,(d,index)->licenses(path+"/"+children[index])).setNegativeButton("Close",null).show();
            } else {
                String text;
                try(InputStream in=getAssets().open(path);ByteArrayOutputStream out=new ByteArrayOutputStream()){
                    copy(in,out);text=out.toString("UTF-8");
                }
                ScrollView scroll=new ScrollView(this);TextView view=label(text,13,Color.WHITE);view.setTextIsSelectable(true);scroll.addView(view);
                new android.app.AlertDialog.Builder(this).setTitle(path.substring(path.lastIndexOf('/')+1)).setView(scroll).setPositiveButton("Close",null).show();
            }
        }catch(IOException ex){report("Cannot open license: "+ex.getMessage());}
    }
    private void pick(){
        if(busy)return;
        Intent i=new Intent(Intent.ACTION_OPEN_DOCUMENT);
        i.addCategory(Intent.CATEGORY_OPENABLE);i.setType("audio/*");startActivityForResult(i,PICK);
    }
    @Override protected void onActivityResult(int req,int result,Intent data){
        super.onActivityResult(req,result,data);
        if(result!=RESULT_OK||data==null||data.getData()==null)return;
        if(req==PICK)load(data.getData());
        if(req==SAVE)export(data.getData());
    }
    private void load(Uri uri){
        if(busy)return;busy=true;stop();report("Loading audio...");
        try(Cursor cursor=getContentResolver().query(uri,new String[]{OpenableColumns.DISPLAY_NAME},null,null,null)){
            if(cursor!=null&&cursor.moveToFirst())name=cursor.getString(0);
        }catch(Exception ignored){}
        new Thread(()->{
            try {
                File file=File.createTempFile("mp3_lab_input_",".audio",getCacheDir());
                try(InputStream in=getContentResolver().openInputStream(uri);
                    OutputStream out=new FileOutputStream(file)){
                    if(in==null)throw new IOException("Cannot open file");
                    copy(in,out);
                }
                MediaMetadataRetriever r=new MediaMetadataRetriever();
                r.setDataSource(file.getAbsolutePath());
                String length=r.extractMetadata(MediaMetadataRetriever.METADATA_KEY_DURATION);r.release();
                double seconds=length==null?0:Double.parseDouble(length)/1000.0;
                if(seconds<=0)throw new IOException("Duration not available");
                handler.post(()->{
                    if(input!=null)input.delete();
                    input=file;duration=seconds;start=0;end=seconds;fadeIn=0;fadeOut=0;gainDb=0;
                    inBar.setProgress(0);outBar.setProgress(1000);
                    fadeInBar.setProgress(0);fadeOutBar.setProgress(0);gainBar.setProgress(667);
                    normalize.setChecked(false);filename.setText(name);refresh();report("Loaded "+name);busy=false;
                });
            }catch(Exception ex){report("Load error: "+ex.getMessage());busy=false;}
        }).start();
    }
    private String quote(String s){return "'"+s.replace("'","'\\''")+"'";}
    private String filters(){
        StringBuilder f=new StringBuilder("asetpts=PTS-STARTPTS");
        if(normalize.isChecked())f.append(",loudnorm=I=-16:TP=-3:LRA=11");
        if(Math.abs(gainDb)>0.001)f.append(String.format(Locale.US,",volume=%+.2fdB",gainDb));
        if(fadeIn>0)f.append(String.format(Locale.US,",afade=t=in:st=0:d=%.3f",fadeIn));
        if(fadeOut>0)f.append(String.format(Locale.US,",afade=t=out:st=%.3f:d=%.3f",end-start-fadeOut,fadeOut));
        return f.append(",alimiter=limit=0.95:attack=5:release=50:level=0").toString();
    }
    private String command(File output,boolean edited){
        StringBuilder b=new StringBuilder(String.format(Locale.US,"-y -nostdin -hide_banner -loglevel error -ss %.5f ",start));
        b.append("-i ").append(quote(input.getAbsolutePath()));
        b.append(String.format(Locale.US," -t %.5f -vn ",end-start));
        if(edited)b.append("-af ").append(quote(filters())).append(' ');
        return b.append("-codec:a libmp3lame -b:a 256k ").append(quote(output.getAbsolutePath())).toString();
    }
    private void preview(boolean edited){
        if(input==null||busy)return;
        if(end-start<.05){report("Choose a longer clip");return;}
        stop();busy=true;report("Preparing preview...");
        int generation=previewGeneration;
        File dest=new File(getCacheDir(),"mp3_lab_preview.mp3");
        FFmpegKit.executeAsync(command(dest,edited),session->{
            if(!ReturnCode.isSuccess(session.getReturnCode())){
                report("Preview encoding failed");busy=false;return;
            }
            handler.post(()->{
                if(generation!=previewGeneration||isDestroyed()){busy=false;return;}
                try{
                    player=new MediaPlayer();player.setDataSource(dest.getAbsolutePath());
                    player.prepare();player.setOnCompletionListener(mp->{stop();report("Preview finished");});
                    player.start();report(edited?"Playing edited preview":"Playing original clip");
                }catch(Exception e){report(e.getMessage());}finally{busy=false;}
            });
        });
    }
    private void stop(){
        previewGeneration++;
        if(player!=null){try{player.stop();}catch(Exception ignored){}player.release();player=null;}
    }
    private void save(){
        if(input==null||busy)return;
        if(end-start<.05){report("Choose a longer clip");return;}
        Intent i=new Intent(Intent.ACTION_CREATE_DOCUMENT);
        i.addCategory(Intent.CATEGORY_OPENABLE);i.setType("audio/mpeg");
        i.putExtra(Intent.EXTRA_TITLE,name.replaceFirst("(?i)\\.[^.]+$","")+"_clip.mp3");
        startActivityForResult(i,SAVE);
    }
    private void export(Uri uri){
        if(input==null||busy)return;
        busy=true;report("Exporting MP3 at 256 kbps...");
        File temp=new File(getCacheDir(),"mp3_lab_export.mp3");
        FFmpegKit.executeAsync(command(temp,true),session->{
            if(!ReturnCode.isSuccess(session.getReturnCode())){report("Encoding failed");busy=false;return;}
            new Thread(()->{
                try(InputStream in=new FileInputStream(temp);
                    OutputStream out=getContentResolver().openOutputStream(uri,"w")){
                    if(out==null)throw new IOException("Output not writable");
                    copy(in,out);report("Saved 256 kbps MP3");
                }catch(Exception e){report("Save failed: "+e.getMessage());}
                finally{busy=false;temp.delete();}
            }).start();
        });
    }
    private static void copy(InputStream in,OutputStream out)throws IOException{
        byte[] data=new byte[65536];int n;
        while((n=in.read(data))!=-1)out.write(data,0,n);
    }
    @Override protected void onDestroy(){stop();super.onDestroy();}
}

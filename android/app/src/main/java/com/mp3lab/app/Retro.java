package com.mp3lab.app;

import android.content.Context;
import android.graphics.*;
import android.graphics.drawable.*;
import android.content.res.ColorStateList;
import android.view.*;
import android.widget.*;

final class Retro {
    static final int LED=0xff72ff00, BLUE=0xff85c6ff, INK=0xff080d0a;
    static GradientDrawable plate(boolean silver){
        GradientDrawable d=new GradientDrawable(GradientDrawable.Orientation.TOP_BOTTOM,silver?
            new int[]{0xffeeeeed,0xffb3b4b4,0xff87898b,0xffbabcbc}:new int[]{0xff383c43,0xff272b31,0xff1b1e24}){
                @Override public void draw(Canvas canvas){super.draw(canvas);Rect b=getBounds();Paint p=new Paint();p.setStrokeWidth(1.5f);
                    p.setColor(silver?0xfffafafa:0xffa0a4ac);canvas.drawLine(b.left+4,b.top+2,b.right-4,b.top+2,p);
                    p.setColor(0xff080a0f);canvas.drawLine(b.left+3,b.bottom-3,b.right-3,b.bottom-3,p);
                    if(!silver){p.setColor(0x09333333);for(int y=b.top+5;y<b.bottom-4;y+=4)for(int x=b.left+5;x<b.right-4;x+=5)if((x*17+y*31)%7<2)canvas.drawPoint(x,y,p);}
                }
            };
        d.setCornerRadius(5);d.setStroke(2,silver?0xffd9dada:0xff727780);return d;
    }
    static void text(TextView v){v.setTextColor(0xfff0f0ed);v.setTypeface(Typeface.create("sans-serif-condensed",Typeface.NORMAL));v.setShadowLayer(1,1,1,Color.BLACK);}
    static void button(Button b){
        b.setBackground(plate(true));b.setTextColor(0xff101216);b.setAllCaps(false);
        b.setTypeface(Typeface.create("sans-serif-condensed",Typeface.BOLD));b.setTextSize(16);b.setMinHeight(0);b.setMinimumHeight(0);
        b.setStateListAnimator(null);b.setPadding(6,4,6,4);
        b.setOnTouchListener((v,e)->{v.setAlpha(e.getAction()==MotionEvent.ACTION_DOWN?.7f:1);return false;});
    }
    static void seek(SeekBar b,int scale){
        b.setProgressTintList(ColorStateList.valueOf(0xff4578ff));
        b.setProgressBackgroundTintList(ColorStateList.valueOf(0xff080b10));
        GradientDrawable thumb=plate(true);thumb.setSize(18*scale,24*scale);b.setThumb(thumb);
        b.setSplitTrack(false);b.setPadding(12*scale,8*scale,12*scale,8*scale);
    }
    interface Selection {void set(double start,double end);}
    static final class Brand extends View {
        final Paint p=new Paint(Paint.ANTI_ALIAS_FLAG);
        Brand(Context c){super(c);setContentDescription("MP3 Lab");}
        @Override protected void onDraw(Canvas c){
            float s=getResources().getDisplayMetrics().density,h=getHeight();
            p.setShader(new LinearGradient(0,0,0,h,0xff19223d,0xff020511,Shader.TileMode.CLAMP));c.drawRect(0,0,getWidth(),h,p);
            p.setShader(new LinearGradient(0,h*.18f,0,h*.9f,new int[]{Color.WHITE,0xffe8e8e8,0xff83878d,0xffc6c7ca},new float[]{0,.45f,.5f,1},Shader.TileMode.CLAMP));
            p.setTypeface(Typeface.create("sans-serif-black",Typeface.BOLD_ITALIC));p.setTextSize(39*s);p.setShadowLayer(2,1,2,Color.BLACK);
            c.drawText("MP3",10*s,h*.8f,p);float x=14*s+p.measureText("MP3");p.clearShadowLayer();
            c.drawText("lab",x+22*s,h*.8f,p);p.setShader(new LinearGradient(x,0,x+16*s,h,0xffffff4e,0xffffbc00,Shader.TileMode.CLAMP));
            Path bolt=new Path();bolt.moveTo(x+27*s,2*s);bolt.lineTo(x+3*s,h*.53f);bolt.lineTo(x+15*s,h*.53f);bolt.lineTo(x-2*s,h-2*s);bolt.lineTo(x+32*s,h*.38f);bolt.lineTo(x+17*s,h*.38f);bolt.close();c.drawPath(bolt,p);p.setShader(null);
        }
    }
    static final class Meter extends View {
        final MainActivity a;final Paint p=new Paint(Paint.ANTI_ALIAS_FLAG);
        Meter(MainActivity activity){super(activity);a=activity;setContentDescription("Source peak level");}
        @Override protected void onDraw(Canvas c){
            float s=getResources().getDisplayMetrics().density,w=getWidth();
            double level=Double.isNaN(a.peakDb)?-60:a.peakDb;
            if(a.player!=null&&a.waveform.length>0&&a.duration>0){int i=(int)(a.waveformView.position/a.duration*a.waveform.length);if(i>=0&&i<a.waveform.length)level=20*Math.log10(Math.max(0.000001,a.waveform[i]));}
            p.setTextSize(10*s);p.setTypeface(Typeface.MONOSPACE);p.setColor(0xffcdd2cf);c.drawText("PEAK",0,12*s,p);
            for(int i=0;i<35;i++){p.setColor(i/35.0<(level+60)/60?(i>30?0xffe8f400:LED):0xff17300d);float x=40*s+i*(w-42*s)/35;c.drawRect(x,3*s,x+(w-42*s)/35-2*s,14*s,p);}
            p.setColor(0xffa6ada9);c.drawText("−60         −36        −12       0 dBFS",0,29*s,p);
        }
    }
    static final class Wave extends View {
        float[] peaks=new float[0];double duration,start,end,position;boolean draggingStart;
        Selection selection;final Paint p=new Paint(Paint.ANTI_ALIAS_FLAG);
        Wave(Context c){super(c);setContentDescription("Audio waveform. Drag the yellow handles to trim the clip.");setFocusable(true);}
        void rect(Canvas c,float l,float t,float r,float b,int color){p.setShader(null);p.setColor(color);c.drawRect(l,t,r,b,p);}
        void label(Canvas c,String s,float x,float y,int color,float size){p.setShader(null);p.setTypeface(Typeface.MONOSPACE);p.setTextSize(size);p.setColor(color);c.drawText(s,x,y,p);}
        @Override protected void onDraw(Canvas c){
            float w=getWidth(),h=getHeight(),s=getResources().getDisplayMetrics().density;
            rect(c,0,0,w,h,0xff060c08);float base=h*.45f;
            for(int i=1;i<8;i++)rect(c,i*w/8,0,i*w/8+1,h-24*s,0xff102719);
            if(duration>0){
                float a=(float)(start/duration*w),b=(float)(end/duration*w);
                p.setShader(new LinearGradient(0,0,0,h,new int[]{0xff124517,0xff167624,0xff08240d},null,Shader.TileMode.CLAMP));
                c.drawRect(a,0,b,h-26*s,p);p.setShader(null);
                p.setColor(LED);p.setStrokeWidth(Math.max(1,w/Math.max(1,peaks.length)));
                for(int i=0;i<peaks.length;i++){
                    float x=(i+.5f)*w/peaks.length,amp=Math.min(1,peaks[i])*base*.92f;
                    c.drawLine(x,base-amp,x,base+amp,p);
                }
                for(float x:new float[]{a,b}){
                    rect(c,x-1.5f*s,0,x+1.5f*s,h-26*s,0xffffe600);
                    rect(c,Math.max(0,x-5*s),0,Math.min(w,x+5*s),10*s,0xffffe600);
                    rect(c,Math.max(0,x-5*s),h-37*s,Math.min(w,x+5*s),h-26*s,0xffffe600);
                }
                if(position>=start&&position<=end)rect(c,(float)(position/duration*w),0,(float)(position/duration*w)+s,h-26*s,Color.WHITE);
            } else label(c,"OPEN AUDIO TO VIEW WAVEFORM",14*s,base,0xff5b8464,11*s);
            rect(c,0,h-26*s,w,h-25*s,0xff979ba0);
            for(int i=0;i<=4;i++){
                float x=i*w/4;rect(c,x,h-26*s,x+s,h-19*s,0xffcacdd0);
                String time=String.format(java.util.Locale.US,"%d:%02d",(int)(duration*i/4)/60,(int)(duration*i/4)%60);
                label(c,time,Math.min(w-42*s,Math.max(2*s,x-12*s)),h-4*s,0xffe4e5e2,11*s);
            }
        }
        @Override public boolean onTouchEvent(MotionEvent e){
            if(duration<=0)return false;
            double t=Math.max(0,Math.min(duration,e.getX()/getWidth()*duration));
            if(e.getAction()==MotionEvent.ACTION_DOWN){draggingStart=Math.abs(t-start)<Math.abs(t-end);getParent().requestDisallowInterceptTouchEvent(true);}
            if(e.getAction()==MotionEvent.ACTION_DOWN||e.getAction()==MotionEvent.ACTION_MOVE){
                if(draggingStart)start=Math.min(t,end);else end=Math.max(t,start);
                if(selection!=null)selection.set(start,end);invalidate();return true;
            }
            if(e.getAction()==MotionEvent.ACTION_UP){getParent().requestDisallowInterceptTouchEvent(false);performClick();return true;}return true;
        }
        @Override public boolean performClick(){super.performClick();return true;}
    }
}

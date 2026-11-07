use super::*;
use egui::{Color32 as C,Vec2,Rect,Stroke,RichText,FontId,Align2};
const GREEN:C=C::from_rgb(105,255,0);
const BLUE:C=C::from_rgb(133,198,255);
const GOLD:C=C::from_rgb(255,229,0);
fn gradient(p:&egui::Painter,r:Rect,top:C,bottom:C){
    let mut m=egui::Mesh::default();
    for (pos,color) in [(r.left_top(),top),(r.right_top(),top),(r.left_bottom(),bottom),(r.right_bottom(),bottom)]{m.colored_vertex(pos,color);}
    m.indices=vec![0,1,2,2,1,3];p.add(egui::Shape::mesh(m));
}
fn panel<R>(ui:&mut egui::Ui,body:impl FnOnce(&mut egui::Ui)->R)->egui::InnerResponse<R>{
    let bg=ui.painter().add(egui::Shape::Noop);
    let result=egui::Frame::new().inner_margin(10).outer_margin(2).corner_radius(5).stroke(Stroke::new(1.0,C::from_gray(119))).show(ui,|ui|{
        ui.with_layout(egui::Layout::top_down(egui::Align::Min),|ui|{ui.set_min_width(ui.available_width());body(ui)}).inner
    });
    let r=result.response.rect;let mut m=egui::Mesh::default();
    for(pos,col) in [(r.left_top(),C::from_rgb(52,56,63)),(r.right_top(),C::from_rgb(52,56,63)),(r.left_bottom(),C::from_rgb(25,28,33)),(r.right_bottom(),C::from_rgb(25,28,33))]{m.colored_vertex(pos,col);}
    m.indices=vec![0,1,2,2,1,3];ui.painter().set(bg,egui::Shape::mesh(m));
    ui.painter().line_segment([r.left_bottom()+egui::vec2(2.0,-2.0),r.right_bottom()+egui::vec2(-2.0,-2.0)],Stroke::new(1.0,C::BLACK));result
}
fn chrome(ui:&mut egui::Ui,text:&str,size:Vec2)->egui::Response{
    let(r,response)=ui.allocate_exact_size(size,egui::Sense::click());
    let press=response.is_pointer_button_down_on();
    ui.painter().rect_filled(r,4.0,C::BLACK);
    gradient(ui.painter(),r.shrink(2.0),if press{C::from_gray(126)}else{C::from_gray(232)},if press{C::from_gray(184)}else{C::from_gray(139)});
    ui.painter().rect_stroke(r.shrink(1.0),3.0,Stroke::new(1.0,if response.hovered(){BLUE}else{C::from_gray(198)}),egui::StrokeKind::Inside);
    ui.painter().line_segment([r.left_bottom()+egui::vec2(3.0,-3.0),r.right_bottom()+egui::vec2(-3.0,-3.0)],Stroke::new(1.0,C::from_gray(64)));
    let center=r.center()+if press{egui::vec2(1.0,1.0)}else{Vec2::ZERO};
    let p=ui.painter();let black=C::from_gray(12);let line=Stroke::new(2.0,black);
    let triangle=|x:f32,direction:f32|{p.add(egui::Shape::convex_polygon(vec![center+egui::vec2(x-direction*7.0,-10.0),center+egui::vec2(x-direction*7.0,10.0),center+egui::vec2(x+direction*9.0,0.0)],black,Stroke::NONE));};
    match text {
        "play"=>triangle(0.0,1.0),
        "pause"=>{for x in [-5.0,5.0]{p.rect_filled(Rect::from_center_size(center+egui::vec2(x,0.0),egui::vec2(5.0,20.0)),0.0,black);}},
        "stop"=>{p.rect_filled(Rect::from_center_size(center,egui::vec2(19.0,19.0)),0.0,black);},
        "back"|"next"=>{let d=if text=="back"{-1.0}else{1.0};triangle(-d*2.0,d);p.line_segment([center+egui::vec2(d*11.0,-10.0),center+egui::vec2(d*11.0,10.0)],Stroke::new(3.0,black));},
        "repeat"|"repeat-on"=>{p.circle_stroke(center,10.0,Stroke::new(2.5,if text=="repeat-on"{C::from_rgb(0,90,5)}else{black}));p.add(egui::Shape::convex_polygon(vec![center+egui::vec2(8.0,-12.0),center+egui::vec2(15.0,-7.0),center+egui::vec2(5.0,-4.0)],black,Stroke::NONE));},
        "close"=>{p.line_segment([center-egui::vec2(8.0,8.0),center+egui::vec2(8.0,8.0)],line);p.line_segment([center+egui::vec2(-8.0,8.0),center+egui::vec2(8.0,-8.0)],line);},
        "maximize"=>{p.rect_stroke(Rect::from_center_size(center,egui::vec2(17.0,17.0)),0.0,line,egui::StrokeKind::Inside);},
        "minimize"=>{p.line_segment([center+egui::vec2(-9.0,7.0),center+egui::vec2(9.0,7.0)],Stroke::new(3.0,black));},
        _=>{let font=FontId::new(18.0,egui::FontFamily::Name("chrome".into()));p.text(center+egui::vec2(0.0,1.0),Align2::CENTER_CENTER,text,font.clone(),C::from_gray(220));p.text(center,Align2::CENTER_CENTER,text,font,black);}
    }response
}
fn heading(ui:&mut egui::Ui,text:&str){ui.label(RichText::new(text).size(18.0).strong().color(BLUE));ui.add_space(5.0);}
fn well<R>(ui:&mut egui::Ui,body:impl FnOnce(&mut egui::Ui)->R)->egui::InnerResponse<R>{egui::Frame::new().fill(C::from_rgb(1,12,5)).stroke(Stroke::new(2.0,C::from_gray(92))).corner_radius(4).inner_margin(12).show(ui,|ui|ui.with_layout(egui::Layout::top_down(egui::Align::Min),body).inner)}
pub fn configure(ctx:&egui::Context){
    let mut fonts=egui::FontDefinitions::default();
    fonts.font_data.insert("arimo".into(),egui::FontData::from_static(include_bytes!("../../assets/fonts/Arimo-Regular.ttf")).into());
    fonts.font_data.insert("arimo-bold".into(),egui::FontData::from_static(include_bytes!("../../assets/fonts/Arimo-Bold.ttf")).into());
    fonts.font_data.insert("arimo-brand".into(),egui::FontData::from_static(include_bytes!("../../assets/fonts/Arimo-BoldItalic.ttf")).into());
    fonts.families.get_mut(&egui::FontFamily::Proportional).unwrap().insert(0,"arimo".into());
    fonts.families.insert(egui::FontFamily::Name("brand".into()),vec!["arimo-brand".into()]);
    fonts.families.insert(egui::FontFamily::Name("chrome".into()),vec!["arimo-bold".into()]);ctx.set_fonts(fonts);
    let mut style=(*ctx.global_style()).clone();let mut v=egui::Visuals::dark();
    v.panel_fill=C::from_rgb(18,21,28);v.window_fill=C::from_rgb(35,39,46);v.extreme_bg_color=C::from_rgb(7,10,14);
    v.override_text_color=Some(C::from_rgb(236,237,232));v.selection.bg_fill=C::from_rgb(39,88,198);v.selection.stroke=Stroke::new(1.0,BLUE);
    for w in [&mut v.widgets.inactive,&mut v.widgets.hovered,&mut v.widgets.active]{w.corner_radius=2.into();w.bg_stroke=Stroke::new(1.0,C::from_gray(103));}
    v.widgets.inactive.bg_fill=C::from_rgb(14,18,23);v.widgets.active.bg_fill=C::from_rgb(48,92,192);v.widgets.hovered.bg_fill=C::from_rgb(50,58,72);
    style.visuals=v;style.spacing.item_spacing=egui::vec2(8.0,7.0);style.spacing.interact_size.y=27.0;style.spacing.slider_width=170.0;
    style.text_styles.insert(egui::TextStyle::Body,FontId::proportional(16.0));style.text_styles.insert(egui::TextStyle::Button,FontId::proportional(16.0));ctx.set_global_style(style);
}
pub fn draw(e:&mut Editor,ui:&mut egui::Ui){
    let ctx=ui.ctx().clone();
    egui::CentralPanel::default().frame(egui::Frame::new().fill(C::from_rgb(13,16,22)).inner_margin(7)).show(ui,|ui|{
      egui::ScrollArea::vertical().show(ui,|ui|{
        let header=panel(ui,|ui|{
            ui.horizontal(|ui|{
                let(r,_)=ui.allocate_exact_size(egui::vec2(300.0,48.0),egui::Sense::hover());
                ui.painter().text(r.left_top()+egui::vec2(0.0,0.0),Align2::LEFT_TOP,"MP3",FontId::new(46.0,egui::FontFamily::Name("brand".into())),C::from_gray(231));
                let x=r.left()+108.0;let y=r.top()-3.0;let bolt=vec![egui::pos2(x+30.0,y),egui::pos2(x,y+32.0),egui::pos2(x+15.0,y+32.0),egui::pos2(x-3.0,y+56.0),egui::pos2(x+39.0,y+20.0),egui::pos2(x+21.0,y+20.0)];
                ui.painter().add(egui::Shape::Path(egui::epaint::PathShape{points:bolt,closed:true,fill:GOLD,stroke:Stroke::new(0.5,C::from_rgb(185,143,0)).into()}));
                ui.painter().text(r.left_top()+egui::vec2(146.0,7.0),Align2::LEFT_TOP,"lab",FontId::new(38.0,egui::FontFamily::Name("brand".into())),C::from_gray(231));
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center),|ui|{
                    if chrome(ui,"close",egui::vec2(39.0,33.0)).on_hover_text("Close").clicked(){ctx.send_viewport_cmd(egui::ViewportCommand::Close);}
                    if chrome(ui,"maximize",egui::vec2(39.0,33.0)).on_hover_text("Maximize / restore").clicked(){let max=ctx.input(|i|i.viewport().maximized.unwrap_or(false));ctx.send_viewport_cmd(egui::ViewportCommand::Maximized(!max));}
                    if chrome(ui,"minimize",egui::vec2(39.0,33.0)).on_hover_text("Minimize").clicked(){ctx.send_viewport_cmd(egui::ViewportCommand::Minimized(true));}
                });
            });
        });
        let drag_rect=Rect::from_min_max(header.response.rect.min,header.response.rect.max-egui::vec2(160.0,0.0));
        if ui.interact(drag_rect,ui.scope_id().with("title-drag"),egui::Sense::drag()).drag_started(){ctx.send_viewport_cmd(egui::ViewportCommand::StartDrag);}
        panel(ui,|ui|{egui::MenuBar::new().ui(ui,|ui|{
            ui.menu_button("File",|ui|{if ui.button("Open audio…").clicked(){ui.close();e.open();}if ui.add_enabled(e.input.is_some(),egui::Button::new("Export MP3…")).clicked(){ui.close();e.export();}});
            ui.menu_button("Edit",|ui|{if ui.button("Select whole track").clicked(){e.start=0.0;e.end=e.duration;e.clamp();ui.close();}if ui.button("Reset effects").clicked(){e.fade_in=0.0;e.fade_out=0.0;e.gain=0.0;e.normalize=false;ui.close();}});
            ui.menu_button("Tools",|ui|{if ui.button("Audio tools / dependencies").clicked(){e.show_tools=true;ui.close();}});
            ui.menu_button("Help",|ui|{if ui.button("About mp3ϟlab").clicked(){e.show_about=true;ui.close();}});
        });});
        ui.horizontal(|ui|{
            let width=ui.available_width();
            ui.allocate_ui(egui::vec2(width*0.64,96.0),|ui|{well(ui,|ui|{
                ui.set_min_width(ui.available_width()-4.0);
                let pos=if e.player.is_some()||e.paused{(e.preview_start+e.elapsed()).min(e.preview_end)}else{e.start};
                ui.label(RichText::new(format!("{} / {}",clock(pos),clock(e.duration))).monospace().size(29.0).color(GREEN));
                let title=e.input.as_ref().and_then(|p|p.file_name()).map(|s|s.to_string_lossy().to_string()).unwrap_or_else(||"NO TRACK LOADED".into());
                ui.label(RichText::new(title).monospace().size(18.0).color(GREEN));
                ui.label(RichText::new("MP3  •  256 kbps export").monospace().color(GREEN));
            });});
            ui.allocate_ui(egui::vec2((width*0.36-12.0).max(100.0),96.0),|ui|{well(ui,|ui|{
                let position=if e.player.is_some(){e.preview_start+e.elapsed()}else{0.0};
                let bin=if e.duration>0.0{((position/e.duration)*e.waveform.len() as f64) as usize}else{0};
                let levels=if e.player.is_some(){e.waveform.get(bin).copied().unwrap_or([0.0;2])}else{[0.0;2]};
                for (channel,level) in [("L",levels[0]),("R",levels[1])]{ui.horizontal(|ui|{
                    ui.label(RichText::new(channel).monospace());let(r,_)=ui.allocate_exact_size(egui::vec2(ui.available_width()-2.0,15.0),egui::Sense::hover());
                    let db=if level>0.0{20.0*level.log10()}else{-60.0};
                    for i in 0..34{let x=r.left()+i as f32*r.width()/34.0;let col=if i as f32/34.0<(db+60.0)/60.0{if i>29{GOLD}else{GREEN}}else{C::from_rgb(20,40,8)};ui.painter().rect_filled(Rect::from_min_size(egui::pos2(x,r.top()),egui::vec2((r.width()/34.0-2.0).max(1.0),15.0)),0.0,col);}
                });}ui.label(RichText::new("−60   −36   −24   −12   0").monospace().size(11.0));ui.small("SOURCE PEAK LEVEL");
            });});
        });
        panel(ui,|ui|{ui.horizontal(|ui|{
            if chrome(ui,"Open…",egui::vec2(84.0,42.0)).clicked(){e.open();}
            ui.add_enabled_ui(e.input.is_some(),|ui|{
                if chrome(ui,"back",egui::vec2(47.0,42.0)).on_hover_text("Back 5 seconds").clicked(){e.seek_preview(-5.0);}
                if chrome(ui,"play",egui::vec2(62.0,42.0)).on_hover_text("Play edited clip").clicked(){if e.paused{e.resume();}else{e.play(true);}}
                if chrome(ui,"pause",egui::vec2(47.0,42.0)).on_hover_text("Pause / resume").clicked(){e.pause();}
                if chrome(ui,"stop",egui::vec2(47.0,42.0)).on_hover_text("Stop").clicked(){e.stop();e.status="Stopped".into();}
                if chrome(ui,"next",egui::vec2(47.0,42.0)).on_hover_text("Forward 5 seconds").clicked(){e.seek_preview(5.0);}
                ui.separator();if chrome(ui,if e.repeat{"repeat-on"}else{"repeat"},egui::vec2(60.0,42.0)).on_hover_text("Repeat clip").clicked(){e.repeat=!e.repeat;}
                if chrome(ui,"Original",egui::vec2(85.0,42.0)).on_hover_text("Preview without effects").clicked(){e.play(false);}
            });
            ui.separator();ui.label("Vol");let response=ui.add(egui::Slider::new(&mut e.volume,0.0..=100.0).show_value(false));
            if response.drag_stopped()&&e.player.is_some(){e.offset=e.elapsed();e.started=None;e.kill_player();e.resume();}
        });});
        panel(ui,|ui|{waveform(e,ui);});
        let loaded=e.input.is_some();
        ui.add_enabled_ui(loaded,|ui|{
          ui.columns(3,|cols|{
            panel(&mut cols[0],|ui|{heading(ui,"Clip");
                egui::Grid::new("clip-values").num_columns(2).spacing([10.0,9.0]).show(ui,|ui|{
                    ui.label("Start:");ui.add(egui::DragValue::new(&mut e.start).speed(0.05).range(0.0..=e.duration).fixed_decimals(3).suffix(" s"));ui.end_row();
                    ui.label("End:");ui.add(egui::DragValue::new(&mut e.end).speed(0.05).range(0.0..=e.duration).fixed_decimals(3).suffix(" s"));ui.end_row();
                    ui.label("Length:");ui.label(RichText::new(clock(e.length())).monospace());ui.end_row();
                });ui.add_space(18.0);
            });e.clamp();let len=e.length();
            panel(&mut cols[1],|ui|{heading(ui,"Fades");
                egui::Grid::new("fades").spacing([10.0,11.0]).show(ui,|ui|{
                    ui.label("Fade In:");ui.add(egui::DragValue::new(&mut e.fade_in).speed(0.05).range(0.0..=len).fixed_decimals(2).suffix(" s"));ui.end_row();
                    ui.label("Fade Out:");ui.add(egui::DragValue::new(&mut e.fade_out).speed(0.05).range(0.0..=len).fixed_decimals(2).suffix(" s"));ui.end_row();
                });ui.add_space(36.0);
            });e.clamp();
            panel(&mut cols[2],|ui|{heading(ui,"Gain");
                ui.checkbox(&mut e.normalize,"Normalize  (−16.0 LUFS)");
                ui.horizontal(|ui|{ui.label("Gain:");ui.add(egui::DragValue::new(&mut e.gain).speed(0.1).range(-24.0..=12.0).fixed_decimals(1).suffix(" dB"));});
                ui.add(egui::Slider::new(&mut e.gain,-24.0..=12.0).show_value(false));
                let peak=e.peak_db.map(|v|format!("Peak: {v:.1} dBFS")).unwrap_or_else(||"Peak: —".into());
                if ui.small_button("Analyze").clicked(){e.status=peak.clone();}ui.label(RichText::new(peak).monospace().color(GREEN));
            });
          });
        });
        panel(ui,|ui|{ui.horizontal(|ui|{
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center),|ui|{
                ui.label(RichText::new("MP3 (256 kbps)").monospace());
                ui.add_enabled_ui(loaded&&e.exporting.is_none(),|ui|{if chrome(ui,"Export MP3…",egui::vec2(205.0,43.0)).clicked(){e.export();}});
                ui.with_layout(egui::Layout::left_to_right(egui::Align::Center),|ui|{ui.label(if e.status.is_empty(){"Ready. Open an audio file."}else{&e.status});});
            });
        });});
      });
    });
    if e.show_about{egui::Window::new("About mp3ϟlab").open(&mut e.show_about).default_width(420.0).show(&ctx,|ui|{
        if e.logo.is_none(){if let Ok(icon)=eframe::icon_data::from_png_bytes(include_bytes!("../../assets/logo.png")){e.logo=Some(ctx.load_texture("brand-logo",egui::ColorImage::from_rgba_unmultiplied([icon.width as usize,icon.height as usize],&icon.rgba),egui::TextureOptions::LINEAR));}}
        if let Some(logo)=&e.logo{ui.add(egui::Image::new(logo).max_width(380.0));}
        ui.label("Local audio editing • MIT licensed • Independent of Winamp");ui.label("Normalize → gain → fades → limiter. Constant 256 kbps MP3 export.");
    });}
    if e.show_tools{egui::Window::new("Audio tools").open(&mut e.show_tools).show(&ctx,|ui|{for n in ["ffmpeg","ffprobe","ffplay"]{ui.label(format!("{n}: {}",tool(n).display()));}ui.label("Windows builds include all three tools. Licenses and source code accompany the release.");});}
}
fn waveform(e:&mut Editor,ui:&mut egui::Ui){
    let(r,response)=ui.allocate_exact_size(egui::vec2(ui.available_width(),166.0),egui::Sense::click_and_drag());
    let wave=Rect::from_min_max(r.min,r.max-egui::vec2(0.0,26.0));ui.painter().rect_filled(wave,4.0,C::from_rgb(0,12,4));
    for i in 1..12{let x=wave.left()+i as f32*wave.width()/12.0;ui.painter().vline(x,wave.y_range(),Stroke::new(0.5,C::from_rgb(15,40,21)));}
    if e.duration>0.0{
        let a=wave.left()+(e.start/e.duration) as f32*wave.width();let b=wave.left()+(e.end/e.duration) as f32*wave.width();
        gradient(ui.painter(),Rect::from_min_max(egui::pos2(a,wave.top()),egui::pos2(b,wave.bottom())),C::from_rgb(16,70,24),C::from_rgb(3,36,10));
        for(i,peaks) in e.waveform.iter().enumerate(){let x=wave.left()+(i as f32+0.5)*wave.width()/e.waveform.len() as f32;let height=peaks[0].max(peaks[1]).min(1.0)*wave.height()*0.46;ui.painter().vline(x,(wave.center().y-height)..=(wave.center().y+height),Stroke::new(1.0,GREEN));}
        for x in [a,b]{ui.painter().vline(x,wave.y_range(),Stroke::new(2.0,GOLD));ui.painter().rect_filled(Rect::from_min_size(egui::pos2((x-3.0).clamp(wave.left(),wave.right()-6.0),wave.top()),egui::vec2(6.0,12.0)),0.0,GOLD);ui.painter().rect_filled(Rect::from_min_size(egui::pos2((x-3.0).clamp(wave.left(),wave.right()-6.0),wave.bottom()-12.0),egui::vec2(6.0,12.0)),0.0,GOLD);}
        if e.player.is_some()||e.paused{let x=wave.left()+((e.preview_start+e.elapsed())/e.duration) as f32*wave.width();ui.painter().vline(x,wave.y_range(),Stroke::new(1.0,C::WHITE));}
        if response.clicked()||response.dragged(){if let Some(pos)=response.interact_pointer_pos(){let t=((pos.x-wave.left())/wave.width()).clamp(0.0,1.0) as f64*e.duration;if (pos.x-a).abs()<(pos.x-b).abs(){e.start=t.min(e.end)}else{e.end=t.max(e.start)}e.clamp();}}
    }else{ui.painter().text(wave.center(),Align2::CENTER_CENTER,"OPEN AUDIO TO VIEW WAVEFORM",FontId::monospace(15.0),C::from_rgb(89,132,98));}
    ui.painter().rect_stroke(wave,4.0,Stroke::new(2.0,C::from_gray(94)),egui::StrokeKind::Inside);
    for i in 0..=7{let x=wave.left()+i as f32*wave.width()/7.0;ui.painter().line_segment([egui::pos2(x,wave.bottom()),egui::pos2(x,wave.bottom()+7.0)],Stroke::new(1.0,C::LIGHT_GRAY));let align=if i==0{Align2::LEFT_TOP}else if i==7{Align2::RIGHT_TOP}else{Align2::CENTER_TOP};ui.painter().text(egui::pos2(x,wave.bottom()+9.0),align,clock(e.duration*i as f64/7.0),FontId::monospace(12.0),C::LIGHT_GRAY);}
}

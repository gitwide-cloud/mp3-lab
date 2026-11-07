"""Exercise the included tools without any developer tools on PATH."""
import json, os, pathlib, subprocess, sys, tempfile
folder=pathlib.Path(sys.argv[1]).resolve()
env=dict(os.environ);env['PATH']=str(pathlib.Path(os.environ['SystemRoot'])/'System32');env['SDL_AUDIODRIVER']='dummy'
def run(tool,*args):
    result=subprocess.run([str(folder/(tool+'.exe')),*map(str,args)],env=env,capture_output=True,text=True,timeout=60)
    assert result.returncode==0,result.stderr
    return result.stdout
for name in ['mp3-lab','ffmpeg','ffprobe','ffplay']:assert (folder/(name+'.exe')).is_file(),name
for name in ['ffmpeg','ffprobe','ffplay']:run(name,'-version')
with tempfile.TemporaryDirectory() as directory:
    output=pathlib.Path(directory)/'test.mp3'
    run('ffmpeg','-y','-f','lavfi','-i','sine=frequency=440:duration=1','-af','loudnorm=I=-16:TP=-3:LRA=11,volume=-3dB,afade=t=in:d=0.1,afade=t=out:st=0.9:d=0.1,alimiter=limit=0.95:level=0','-c:a','libmp3lame','-b:a','256k',output)
    info=json.loads(run('ffprobe','-v','error','-show_streams','-of','json',output))
    assert info['streams'][0]['codec_name']=='mp3' and info['streams'][0]['bit_rate']=='256000'
    run('ffplay','-nodisp','-autoexit','-loglevel','error',output)
print('Bundled probe, normalization, gain, fades, limiter, 256 kbps MP3 and SDL playback passed with system-only PATH.')

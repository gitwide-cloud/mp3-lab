"""Download public release assets and verify checksums plus archive contents."""
import hashlib, json, pathlib, sys, urllib.request, zipfile, tarfile
repo='https://api.github.com/repos/gitwide-cloud/mp3-lab'
tag=sys.argv[1]
with urllib.request.urlopen(repo+'/releases/tags/'+tag) as response:release=json.load(response)
assert not release['draft']
out=pathlib.Path(__file__).resolve().parents[1]/'dist'/tag;out.mkdir(parents=True,exist_ok=True)
for asset in release['assets']:
    dest=out/asset['name']
    urllib.request.urlretrieve(asset['browser_download_url'],dest)
    assert dest.stat().st_size==asset['size']>0
for line in (out/'SHA256SUMS.txt').read_text().splitlines():
    expected,name=line.split(maxsplit=1);name=name.strip()
    assert hashlib.sha256((out/name).read_bytes()).hexdigest()==expected,name
for platform,magic in [('windows',b'MZ'),('linux',b'\x7fELF')]:
    with zipfile.ZipFile(out/f'mp3-lab-{platform}.zip') as z:
        assert z.testzip() is None
        exe='mp3-lab.exe' if platform=='windows' else 'mp3-lab'
        assert z.read(exe).startswith(magic)
        assert len(json.loads(z.read('licenses/dependencies.json')))>10
        assert z.read('LICENSE') and z.read('Cargo.lock')
        if platform=='windows':
            for tool in ['ffmpeg.exe','ffprobe.exe','ffplay.exe']:assert z.read(tool).startswith(b'MZ')
            assert z.read('mp3-lab.ico') and z.read('licenses/Arimo-OFL.txt')
assert (out/'mp3-lab-setup.exe').read_bytes().startswith(b'MZ')
with tarfile.open(out/'windows-audio-sources.tar.gz') as tar:
    assert tar.extractfile('sources/build_windows_audio.sh').read()
    for name in ['ffmpeg-n8.1.3.tar.gz','SDL2-2.32.10.tar.gz','lame-3.100.tar.gz']:
        assert tar.getmember('sources/'+name).size>1000
with zipfile.ZipFile(out/'mp3-lab.apk') as z:
    assert z.testzip() is None
    assert z.read('classes.dex').startswith(b'dex')
    assert z.read('lib/arm64-v8a/libffmpegkit.so').startswith(b'\x7fELF')
    assert not any(n.startswith('lib/x86') for n in z.namelist())
    for name in ['LGPL-3.0.txt','GPL-3.0.txt','smart-exception-java-BSD-3-Clause.txt','smart-exception-common-BSD-3-Clause.txt']:
        assert z.read('assets/licenses/'+name)
with zipfile.ZipFile(out/'android-third-party-sources.zip') as z:
    assert z.testzip() is None
    assert len(json.loads(z.read('SOURCES.json')))>=16
print(json.dumps({'release':release['html_url'],'tag':tag,'verified_assets':[{ 'name':a['name'],'size':a['size'],'url':a['browser_download_url']} for a in release['assets']],'directory':str(out)},indent=2))

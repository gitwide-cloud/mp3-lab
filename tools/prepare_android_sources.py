"""Preserve pinned upstream sources, build scripts and license notices for redistribution."""
import concurrent.futures, hashlib, io, json, pathlib, re, tarfile, urllib.request, zipfile

ROOT=pathlib.Path(__file__).resolve().parents[1]
OUT=ROOT/'dist'; OUT.mkdir(exist_ok=True)
ASSETS=ROOT/'android/app/src/main/assets/licenses'; ASSETS.mkdir(parents=True,exist_ok=True)
PIN='469e560b89a9ba4cab357d2c306c676b0c611745'
def fetch(url):
    with urllib.request.urlopen(url,timeout=180) as r: return r.read()
wrapper=fetch(f'https://codeload.github.com/ffmpegkit-maintained/ffmpeg/tar.gz/{PIN}')
tar=tarfile.open(fileobj=io.BytesIO(wrapper),mode='r:gz')
def member(suffix):
    return tar.extractfile(next(x for x in tar.getmembers() if x.name.endswith('/'+suffix))).read()
pins=member('android-8.1-lts/scripts/source.sh').decode()
names=['config','cpu-features','ffmpeg','lame','libilbc','libogg','libvorbis','opencore-amr','opus','shine','soxr','speex','twolame','vo-amrwbenc']
def source(name):
    block=re.search(r'^  '+re.escape(name)+r'\)\n(.*?)    ;;',pins,re.M|re.S).group(1)
    repo=re.search(r'SOURCE_REPO_URL="https://github.com/([^"]+)"',block).group(1)
    ref=re.search(r'SOURCE_ID="([^"]+)"',block).group(1)
    url=f'https://codeload.github.com/{repo}/tar.gz/{ref}'
    data=fetch(url)
    return name,url,data
items=list(concurrent.futures.ThreadPoolExecutor(max_workers=4).map(source,names))
# Exact upstream wrapper and build scripts, external library sources, plus Java helper sources.
items.append(('ffmpegkit-'+PIN,f'https://github.com/ffmpegkit-maintained/ffmpeg/tree/{PIN}',wrapper))
smart_sources=[]
for artifact in ['smart-exception-java','smart-exception-common']:
    url=f'https://repo.maven.apache.org/maven2/com/arthenica/{artifact}/0.2.1/{artifact}-0.2.1-sources.jar'
    data=fetch(url);smart_sources.append((artifact,url,data))
    with zipfile.ZipFile(io.BytesIO(data)) as jar:
        header=next(jar.read(n).decode().split('*/')[0]+'*/' for n in jar.namelist() if n.endswith('.java'))
        (ASSETS/(artifact+'-BSD-3-Clause.txt')).write_text(header,encoding='utf-8')
# The AAR also redistributes the NDK libc++ runtime. Preserve LLVM's runtime notices.
runtime_notices=[]
for component in ['libcxx','libcxxabi','libunwind']:
    url=f'https://raw.githubusercontent.com/llvm/llvm-project/llvmorg-18.1.8/{component}/LICENSE.TXT'
    data=fetch(url);runtime_notices.append((component,url,data))
    (ASSETS/(component+'-LICENSE.txt')).write_bytes(data)
index=[]
with zipfile.ZipFile(OUT/'android-third-party-sources.zip','w',zipfile.ZIP_DEFLATED) as z:
    for name,url,data in items:
        z.writestr(name+'.tar.gz',data)
        index.append({'name':name,'url':url,'sha256':hashlib.sha256(data).hexdigest()})
        with tarfile.open(fileobj=io.BytesIO(data),mode='r:gz') as t:
            for m in t.getmembers():
                if m.isfile() and any(m.name.split('/')[-1].upper().startswith(p) for p in ['LICENSE','COPYING','NOTICE','COPYRIGHT','AUTHORS']):
                    # Preserve notices in the APK as well as the source archive.
                    dest=ASSETS/name/pathlib.Path(*m.name.split('/')[1:]);dest.parent.mkdir(parents=True,exist_ok=True)
                    dest.write_bytes(t.extractfile(m).read())
    for name,url,data in smart_sources:
        z.writestr(name+'-0.2.1-sources.jar',data)
        index.append({'name':name,'url':url,'sha256':hashlib.sha256(data).hexdigest()})
    for name,url,data in runtime_notices:
        z.writestr('runtime-notices/'+name+'-LICENSE.txt',data)
    z.writestr('SOURCES.json',json.dumps(index,indent=2))
    z.write(ROOT/'THIRD_PARTY_NOTICES.md','THIRD_PARTY_NOTICES.md')
(ASSETS/'LGPL-3.0.txt').write_bytes(member('LICENSE'))
(ASSETS/'GPL-3.0.txt').write_bytes(member('android-8.1-lts/tools/license/LICENSE.GPLv3'))
(ASSETS/'MP3-Lab-MIT.txt').write_bytes((ROOT/'LICENSE').read_bytes())
(ASSETS/'THIRD_PARTY_NOTICES.md').write_bytes((ROOT/'THIRD_PARTY_NOTICES.md').read_bytes())
print('Prepared Android notices and',OUT/'android-third-party-sources.zip')

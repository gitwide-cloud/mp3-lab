import json, pathlib, subprocess, sys, zipfile
root=pathlib.Path(__file__).resolve().parents[1]
platform=sys.argv[1]
metadata=json.loads(subprocess.check_output(['cargo','metadata','--format-version','1'],cwd=root/'desktop'))
with zipfile.ZipFile(root/f'mp3-lab-{platform}.zip','w',zipfile.ZIP_DEFLATED,strict_timestamps=False) as z:
    binary='mp3-lab.exe' if platform=='windows' else 'mp3-lab'
    z.write(root/'desktop/target/release'/binary,binary)
    for name in ['README.md','LICENSE','THIRD_PARTY_NOTICES.md']:z.write(root/name,name)
    z.write(root/'desktop/Cargo.lock','Cargo.lock')
    index=[]
    for p in metadata['packages']:
        base=pathlib.Path(p['manifest_path']).parent
        index.append({'name':p['name'],'version':p['version'],'license':p['license'],'repository':p['repository']})
        for f in base.rglob('*'):
            if f.is_file() and any(f.name.upper().startswith(s) for s in ['LICENSE','COPYING','NOTICE','COPYRIGHT']):
                z.write(f,f"licenses/{p['name']}-{p['version']}/{f.relative_to(base).as_posix()}")
    z.writestr('licenses/dependencies.json',json.dumps(index,indent=2))

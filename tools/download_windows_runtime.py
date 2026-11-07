"""Reuse the successful source build only when its build script matches this checkout."""
import hashlib, json, os, pathlib, subprocess, tarfile, time
root=pathlib.Path(__file__).resolve().parents[1]
expected=(root/'tools/build_windows_audio.sh').read_bytes().replace(b'\r\n',b'\n')
seen=set()
for attempt in range(40):
    runs=json.loads(subprocess.check_output(['gh','api','repos/gitwide-cloud/mp3-lab/actions/workflows/windows-audio.yml/runs?status=success&per_page=3']))['workflow_runs']
    for run in runs:
        run_id=run['id']
        if run_id in seen:continue
        seen.add(run_id)
        directory=root/'dist'/f'audio-{run_id}';directory.mkdir(parents=True,exist_ok=True)
        subprocess.run(['gh','run','download',str(run_id),'-R','gitwide-cloud/mp3-lab','-n','windows-audio-sources','-D',str(directory)],check=True)
        source=directory/'windows-audio-sources.tar.gz'
        with tarfile.open(source) as archive:actual=archive.extractfile('sources/build_windows_audio.sh').read().replace(b'\r\n',b'\n')
        if actual!=expected:continue
        subprocess.run(['gh','run','download',str(run_id),'-R','gitwide-cloud/mp3-lab','-n','windows-audio-runtime','-D',str(root/'windows-runtime')],check=True)
        import shutil
        shutil.copy2(source,root/'dist/windows-audio-sources.tar.gz')
        print('Verified Windows runtime build',run_id,'script SHA256',hashlib.sha256(actual).hexdigest());raise SystemExit(0)
    print('Waiting for matching Windows runtime source build...',flush=True);time.sleep(30)
raise SystemExit('No successful matching Windows audio runtime build within 20 minutes')

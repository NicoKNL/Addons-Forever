#!/usr/bin/env python3
"""Independent verification of the CurseForge folder fingerprint algorithm
(MurmurHash2-32, seed=1, whitespace-stripped) against known module hashes:
BigWigs v427.8 -> modules [3793833344, 872682054, 895199333, 908271451]."""
import json, re, sys, urllib.request, zipfile, io, os

M = 0x5BD1E995
KEY = "$2a$10$bL4bIL5pUWqfcO7KQtnMReakwtfHbNKh6v1uTpKlzhwoueEJQnPnm"
STRIP = {0x20, 0x09, 0x0A, 0x0D}

def murmur2(data: bytes, seed: int = 1) -> int:
    h = (seed ^ len(data)) & 0xFFFFFFFF
    i = 0; shift = 0
    for b in data:
        i |= b << shift; shift += 8
        if shift == 32:
            i = (i * M) & 0xFFFFFFFF
            i ^= i >> 24
            i = (i * M) & 0xFFFFFFFF
            h = (h * M) & 0xFFFFFFFF
            h ^= i
            i = 0; shift = 0
    if shift > 0:
        h ^= i
        h = (h * M) & 0xFFFFFFFF
    h ^= h >> 13
    h = (h * M) & 0xFFFFFFFF
    h ^= h >> 15
    return h

def file_hash(data: bytes) -> int:
    return murmur2(bytes(b for b in data if b not in STRIP), 1)

TOC_INCLUDE = re.compile(r'^\s*((?:(?!\.\.).)+\.(?:xml|lua))\s*$', re.I)
TOC_COMMENT = re.compile(r'^\s*#.*$')
XML_COMMENT = re.compile(r'<!--.*?-->', re.S)
XML_INCLUDE = re.compile(r'(?i)<(?:Include|Script)\s+file=["\']((?:(?!\.\.).)+)["\']\s*/>')

def walk(name, files, seen):
    """Recursively collect files hashed for one addon folder (files is a dict path->bytes)."""
    lower = {k.lower(): k for k in files}
    def resolve(rel, base):
        p = os.path.normpath(os.path.join(base, rel)).replace('\\','/').lower().lstrip('/')
        return lower.get(p)
    def visit(path):
        if path is None or path in seen: return
        seen.add(path)
        data = files[path]
        ext = path.rsplit('.',1)[-1].lower()
        if ext == 'toc':
            text = data.decode('utf-8-sig', 'replace')
            for line in text.splitlines():
                if TOC_COMMENT.match(line): continue
                m = TOC_INCLUDE.match(line.split('#',1)[0].strip())
                if m: visit(resolve(m.group(1), os.path.dirname(path)))
        elif ext == 'xml':
            text = XML_COMMENT.sub(' ', data.decode('utf-8-sig','replace'))
            for m in XML_INCLUDE.finditer(text):
                visit(resolve(m.group(1), os.path.dirname(path)))
    base = name
    # top-level toc(s) for this module folder
    for k in files:
        if os.path.dirname(k) == base and k.lower().endswith('.toc'):
            bname = os.path.basename(k)[:-4]
            if bname.lower().split('_')[0] == base.split('_')[0].lower() and base.count('/') == 0:
                visit(k)
    b = resolve('Bindings.xml', base)
    if b: visit(b)
    hashes = sorted(file_hash(files[k]) for k in seen)
    joined = ''.join(str(h) for h in hashes).encode()
    return murmur2(joined, 1)

def http(url, data=None, headers=None):
    req = urllib.request.Request(url, data=data, headers=headers or {})
    return urllib.request.urlopen(req, timeout=60).read()

def main():
    # 1) get BigWigs v427.8 (mod 2382, file 9299959) download url + expected module fingerprints
    meta = json.loads(http("https://api.curseforge.com/v1/mods/2382/files/9299959", headers={"x-api-key": KEY}))["data"]
    print("file:", meta["fileName"], "modules:", [m["fingerprint"] for m in meta["modules"]])
    dl = meta["downloadUrl"]
    zip_bytes = http(dl, headers={"x-api-key": KEY})
    zf = zipfile.ZipFile(io.BytesIO(zip_bytes))
    files = {n: zf.read(n) for n in zf.namelist() if not n.endswith('/')}
    print("zip entries:", len(files))
    # find top-level module dirs that contain a toc matching dir name
    tops = sorted({n.split('/')[0] for n in files if n.count('/') >= 1 and n.split('/')[0].upper() != '__MACOSX'})
    results = {}
    for t in tops:
        seen = set()
        results[t] = walk(t, files, seen)
    print("computed:", json.dumps(results, indent=1))
    fps = sorted(results.values())
    expected = sorted([m["fingerprint"] for m in meta["modules"]])
    print("expected:", expected)
    print("MATCH:", fps == expected)
    # 2) fingerprint API round-trip
    body = json.dumps({"fingerprints": fps}).encode()
    resp = json.loads(http("https://api.curseforge.com/v1/fingerprints/1", data=body,
        headers={"x-api-key": KEY, "Content-Type": "application/json"}))
    d = resp["data"]
    print("api exact:", [m["id"] for m in d.get("exactMatches", [])],
          "partial:", [m["id"] for m in d.get("partialMatches", [])],
          "partialMap:", list(d.get("partialMatchFingerprints", {}).keys()))

if __name__ == '__main__':
    main()

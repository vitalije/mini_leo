import os
import subprocess
import zipfile
import hashlib
import base64
import sys
from distutils.util import get_platform
PLATFORM = get_platform().replace('-', '_').replace('.', '_')
def cargo_build():
    proc = subprocess.Popen('cargo build --lib --release',
        cwd='rust')
    proc.wait()
METADATA = b'''
Metadata-Version: 2.1
Name: mini_leo
Version: 0.1.0
Summary: A minimal version of Leo editor.
Home-page: https://github.com/vitalije/mini_leo
Author: Vitalije Milosevic
Author-email: vitalije@kviziracija.net
License: MIT license
Keywords: mini_leo
Platform: any
Classifier: Development Status :: 2 - Pre-Alpha
Classifier: Intended Audience :: Developers
Classifier: License :: OSI Approved :: MIT License
Classifier: Natural Language :: English
Classifier: Programming Language :: Python :: 3
Classifier: Programming Language :: Python :: 3.6
Classifier: Programming Language :: Python :: 3.7
Classifier: Programming Language :: Python :: 3.8
'''.strip()
WHEEL = b'''
Wheel-Version: 1.0
Root-Is-Purelib: false
Tag: py3-none-%s
'''%(PLATFORM.encode('utf8')).strip()
def getversion():
    return METADATA.partition(b'\nVersion:')[2].strip().partition(b'\n')[0].decode('utf8')
def make_wheel():
    if sys.platform == 'linux':
        makelinux_wheel()
    elif sys.platform == 'win32':
        makewin_wheel()
def makelinux_wheel():
    s = open('rust/target/release/libmini_leo.so', 'rb').read()
    makeany_wheel('mini_leo/_minileo.so', s)
def makewin_wheel():
    s = open('rust/target/release/mini_leo.dll', 'rb').read()
    makeany_wheel('mini_leo/_minileo.pyd', s)
def makeany_wheel(dllname, dllcont):
    ver = getversion()
    zf = zipfile.ZipFile('dist/mini_leo-%s-py3-none-%s.whl'%(ver, PLATFORM), 'w')
    def fline(f, cont):
        return '%s,sha256=%s,%d'%(
                f,
                base64.urlsafe_b64encode(hashlib.sha256(cont).digest()).rstrip(b'=').decode('utf8'),
                len(cont)
            )
    buf = []
    def addf(f, cont):
        buf.append(fline(f, cont))
        zf.writestr(f, cont)
    addf(dllname, dllcont)
    addf('mini_leo/__init__.py', open('mini_leo/__init__.py', 'rb').read())
    dinfo = 'mini_leo-%s.dist-info/'%ver
    addf(dinfo + 'METADATA', METADATA)
    buf.append(dinfo + 'RECORD,,')
    addf(dinfo + 'WHEEL', WHEEL)
    zf.writestr(dinfo + 'RECORD', '\n'.join(buf).encode('utf8'))
    zf.close()
if __name__ == '__main__':
    if not os.path.exists('dist'):
        os.makedirs('dist')
    make_wheel()

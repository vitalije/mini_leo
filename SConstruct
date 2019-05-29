# Starter SConstruct for enscons

import enscons
import pytoml as toml
import os

metadata = dict(toml.load(open('pyproject.toml')))['tool']['enscons']

full_tag = enscons.get_binary_tag()

env = Environment(tools=['default', 'packaging', enscons.generate],
                  PACKAGE_METADATA=metadata,
                  WHEEL_TAG=full_tag,
                  ROOT_IS_PURELIB=full_tag.endswith('-any'),
                  ENV=os.environ)

# Only *.py is included automatically by setup2toml.
# Add extra 'purelib' files or package_data here.
py_source = Glob('mini_leo/*.py')

rust_libname = 'mini_leo${SHLIBSUFFIX}'
rust_lib = 'rust/target/release/${SHLIBPREFIX}' + rust_libname

# Build rust
env.Command(
        target=rust_lib,
        source=["rust/Cargo.toml", "rust/build.rs", "rust/src/lib.rs"],
        action="~/.cargo/bin/cargo build --release", 
        chdir="rust"
        )
# Copy compiled library into base directory
local_rust = env.Command(
        target=rust_libname,
        source=rust_lib,
        action=Copy('$TARGET', '$SOURCE'))

local_rust_h = ['rust/target/mini_leo.h']
wheelfiles = env.Whl('platlib', py_source + local_rust + local_rust_h, root='')
whl = env.WhlFile(source=wheelfiles)

# Add automatic source files, plus any other needed files.
sdist_source=FindSourceFiles() + [
    'PKG-INFO',
    'setup.py',
    'LICENSE',
    'README.md',
    'CONTRIBUTING.md',
    'AUTHORS.md',
    'HISTORY.md',
    ]
sdist_source = [x for x in sdist_source if not str(x) == os.path.normpath(local_rust_h[0])]
sdist = env.SDist(source=sdist_source)

env.NoClean(sdist)
env.Alias('sdist', sdist)

# needed for pep517 / enscons.api to work
env.Default(whl, sdist)

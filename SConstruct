#*#*#*#*#*#*#*#*#*#*#*#
#*#*# Cooper IGVC #*#*#
#*#*#*#*#*#*#*#*#*#*#*#

import os

# Basic setup ---------------------------------------------
# We should get more disciplined about our PATH later.
env = Environment(ENV = {'PATH' : os.environ['PATH']})

# Save the repo root in the env
env['REPO_ROOT'] = env.Dir('.')

Decider('content-timestamp')

term = os.environ.get('TERM') # for color
if term is not None:
    env['ENV']['TERM'] = term
# ---------------------------------------------------------


# Global help adder function ------------------------------
help_list = []

def AddHelp(cmd, text):
    global help_list
    help_list.append((cmd, text))

env['AddHelp'] = AddHelp
# ---------------------------------------------------------


# Cleaning targets ----------------------------------------
[rm_build] = env.Command(
    'phony-rm-build',
    [],
    'rm -rf build/'
)

[rm_deps] = env.Command(
    'phony-rm-deps',
    [],
    'rm -rf deps/'
)

env.Alias('clean',    rm_build)
env.Alias('cleanall', [rm_build, rm_deps])
AddHelp('clean',    'Clean (remove) build/ directory')
AddHelp('cleanall', 'Clean (remove) build/ and deps/ (aka everything)')
# ---------------------------------------------------------


# Call SConscripts ----------------------------------------
Default(None)
Export('env')

# Dependencies first
env.SConscript('dependencies.SConscript', variant_dir='deps',              duplicate=0)
env.SConscript('can/SConscript',          variant_dir='build/can',         duplicate=0)
env.SConscript('fw/SConscript',           variant_dir='build/fw', duplicate=0)
# ---------------------------------------------------------

# Populate Help -------------------------------------------
# scons provides Help for you to call to provide the text given by `scons -h`.
# you can call Help more than once and it will append.
Help('''
     So you want to build a car?

     You can specify targets after `scons`, like:

''')

help_list.sort()

for (cmd, text) in help_list:
    Help(f"     `scons {cmd + '`' : <30} {text : <60}\n")

Help(f'''

     Note: try these helpful aliases (if you have `direnv`):

     `fwpio`    Equivalent to `pio`, but specifically for fw/, can
                be used anywhere in the repo, and uses scons. This is the
                recommended way to use PlatformIO. For example:

                    $ fwpio run -e blink1.1
''')
# ---------------------------------------------------------

# shell ---------------------------------------------------
if 'shell' in COMMAND_LINE_TARGETS:
    import pathlib
    import psutil

    parent_shell_path = psutil.Process().parent().exe()
    parent_shell = pathlib.PurePath(parent_shell_path).name

    if parent_shell in ['bash', 'fish', 'sh', 'zsh']:
        print("* Dropping you into a shell...")
        env['ENV']['SCONS_SHELL'] = '1' # just a marker in case anyone needs it
        # propagate entire external env merged with env['ENV']
        env['ENV'] = {**os.environ, **env['ENV']}
        env.Execute(parent_shell_path)
    else:
        print(f"Won't execute shell for unsupported parent process {parent_shell}!")
        print("Try whitelisting it in the SConscript.")
        exit(-1)

# we want to execute the shell first, so we did that above.
# alias 'shell' to None so scons doesn't try to build anything
env.Alias('shell', None)

# ---------------------------------------------------------

if not COMMAND_LINE_TARGETS:
    from SCons.Script import help_text
    print(help_text)
    exit(0)

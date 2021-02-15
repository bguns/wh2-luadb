# wh2-luadb
A modding tool for Total War: Warhammer 2 that can automatically generate Lua data tables from game DB files. When used as a tool, it can do so for specific packfiles or extracted DB files. It can also be used in conjunction with [Kaedrin's Mod Manager](https://github.com/Kaedrin/warhammer-mod-manager) to automatically generate these Lua scripts for all currently active mods when a user launches the game. It uses [RPFM](https://github.com/Frodo45127/rpfm)'s library to manage packfiles and their contents.

## Installation for players
First, make sure you are subscribed to the companion mod for this tool, found <here - WIP, should be finished soon>.

1. Download the latest binaries from the [Releases](https://github.com/bguns/wh2-luadb/releases) page.
2. Unzip both executables to the same directory as Kaedrin's Mod Manager (i.e. the directory that contains Warhammer2MM.exe)
3. (Optional) Create a shortcut to wh2-luadb-kmm-launcher.exe (right click -> Create Shortcut) and place it somewhere convenient.
4. Launch Kaedrin's Mod Manager by executing wh2-luadb-kmm-launcher.exe (or the shortcut you created in step 3).
5. It's possible that Windows warns you about executing an untrusted executable. This is normal. Click on "More information" and then "Run anyway". You should only need to do this the first time you run the launcher.
6. If your Total War: Warhammer 2 installation directory is under C:\\Program Files (x86)\\..., it might be necessary to run wh2-luadb-kmm-launcher.exe as Administrator. To do so, create a shortcut (as in step 3, if you have not already done so), then right click that shortcut, go to Properties, then Advanced, and tick the "Run as Administrator" box.
7. Don't forget to activate the LuaDB mod in Kaedrin's Mod Manager before starting the game.

## Usage for mod creators
For mod creators, the basic usage is through the LuaDB mod, available <here - WIP, should be finished soon>, which provides functions that allow you to load and access the generated data tables from your Lua scripts.

## Use as a standalone command line tool
It is possible to use wh2-luadb.exe as a command line tool to generate Lua tables from a selected packfile's DB files, or from a folder containing extracted DB files. You can use the tool in this way from wherever, it doesn't have to be located in your Warhammer 2 install directory, or in the KMM directory.
### Command line options:
* --packfile, -p FILE\_PATH: with FILE\_PATH pointing to a .pack file, this option will generate Lua table scripts for all DB files found in the selected packfile. If no output directory is specified, the output will be a directory with the same name as the packfile.
* --indir, -i DIRECTORY\_PATH: with DIRECTORY\_PATH pointing to a directory in which you have previously extracted DB files from RPFM, this will look in DIRECTORY\_PATH\\db\\<table_folders\> for DB files to generate Lua tables from. If no output directory is specified, the output will be the same as the input directory.
* --outdir, -o DIRETOCTY\_PATH: with DIRECTORY\_PATH pointing to an *empty* directory. Generated files will be placed in this directory.
* --force: Normally, the output directory should be empty or the program will terminate in order not to accidentally overwrite anything. If you know what you are doing, however, you can use this option to ignore this behaviour, and the program will happily dump all generated files in the output directory without any checks or balances.
* --unpacked, -u: The default behaviour is to generate a file called "lua\_db\_generated.pack" in the output directory. This is a (movie-type) packfile containing all the generated Lua scripts. Using this option, the scripts will instead be written to disk directly, in the same directory structure they would have in the generated packfile's "script" directory.
* --base and --core-prefix: These two options deal with "data coring", meaning when a mod includes a DB file called "data__". Such files will entirely overwrite the base game's DB before other mods are applied on top of it, and two mods doing this for the same DB table is an almost guaranteed mod conflict. To deal with this tricky issue on the Lua-side of things, the tool provides essentially three options:
    * Use the --packfile option and neither of these two options: the generated Lua scripts for any data-cored DB files will be named "<packfile\_name\>\_data__.lua", and they will be placed under <out\_dir\>\\lua\_db\\mod\_core\\<table\_name\>\\
    * Use the --core-prefix <PREFIX\> option: as above, but the generated Lua scripts will be named "<PREFIX\>\_data__.lua"
    * Use the --base option: The generated Lua scripts for data-cored DB tables will simply be called "data__.lua", and they will be placed under <out\_dir\>\\lua\_db\\core\\<table\_name\>\\. I use this option to generate the scripts for the vanilla game data in the LuaDB mod.
* --script-check, -s <VFS\_SCRIPT\_PATH\>: Since users can now generate all the Lua data based on which mods they have loaded, this option is somewhat obsolete. If present, it will add a conditional check to all the generated Lua scripts, so that they only return data if the <VFS\_SCRIPT\_PATH\> is actually present on the VFS ingame. It allows for conditional loading based on whether or not a certain mod is loaded, for example, and was intended as a tool for compatibility modding. Again, however, now that all relevant data can be generated on game start, it is probably best to rely on that for compatibility.
# shad3

shad3 is an application using SHA3-512 to hash entries in a list or multiple lists.

For multiple lists, simply provide the highest numbered file's location/URL.

Use --expect "prefix goes here" after the file location/URL to only hash texts following the given prefix.

## Example: 

### Linux:
./shad3 http://blocklists.io/block04.txt --expect "127.0.0.1 "

### MacOS:
./shad3 http://blocklists.io/block04.txt --expect "127.0.0.1 "

### Windows:
shad3.exe http://blocklists.io/block04.txt --expect "127.0.0.1 "

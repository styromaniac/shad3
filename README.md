# SHAD3

SHAD3 is an application using SHA3-512 to hash entries in a list or multiple lists.

For multiple lists, simply provide the highest numbered file's location/URL.

Use --expect "prefix goes here" after the file location/URL to only hash texts following the given prefix.

## Example: 
```
./shad3 http://blocklists.io/block04.txt --expect "127.0.0.1 "

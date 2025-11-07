#!/usr/bin/env -S awk -f

# push to end of an array
function push(array, value) {
  array[length(array) + 1] = value;
}

BEGIN {
  # This constructs an empty array. Don't ask.
  delete global_section;
  delete export_section;
}

# match against defines which have a value that is a dec or hex integer literal
"#define" == $1 && $2 ~/^SYSTEM_LIMIT_\w+$/ && $3 ~/^-?([0-9]+|0x[0-9a-fA-F]+)$/ {
  # push global declaration
  push(global_section, "(global $" $2 " i64 (i64.const " $3 "))");

  # push global export
  push(export_section, "(export \"" $2 "\" (global $" $2 "))");
}

END {
  printf("(module")

  # print the globals
  for (i in global_section)
    printf("  %s\n", global_section[i]);

  # print the exports
  for (i in global_section)
    printf("  %s\n", export_section[i]);

  print ")"
}


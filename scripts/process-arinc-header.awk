#!/usr/bin/env -S awk -f

BEGIN {
  # retain information whether the current section of the header is portable or not
  is_impl_dependent = -1;

  # use linux style linendings (just \n, not \r\n)
  # ORS = "\n";

  BINMODE = 0;

  # make sure integer types with guaranteed bit width are available
  print "#include <stdint.h>"

  print("#ifdef  __wasm__")
  print("#define  WASM_IMPORT_MODULE(module, name)  __attribute__((import_module(module), import_name(name)))")
  print("#else")
  print("#define  WASM_IMPORT_MODULE(module, name)")
  print("#endif")
}

# detect that we are in an implementation dependent section of the header
"/*" == $1 && "Implementation" == $2 && "Dependent" == $3 {
  is_impl_dependent = 1;
}

# detect that we are in an implementation portable section of the header
"/*" == $1 && "Implementation" == $2 && "Portable" == $3 {
  is_impl_dependent = 0;
}

# mark all functions to be importend from the arinc module
"extern" == $1 && "void" == $2 && $4 ~/^\(/ {
  import_module = "arinc653:p1@0.1.0";
  print "WASM_IMPORT_MODULE(\"" import_module "\", \"" $3 "\")"
}

# make all function pointers actually be function pointers
$0 ~/SYSTEM_ADDRESS_TYPE\s+ENTRY_POINT/ {
  gsub(/SYSTEM_ADDRESS_TYPE\s+ENTRY_POINT/,"void (*ENTRY_POINT)(void)", $0);
}

# make all implementation dependent defines ifndef based
"#define" == $1 && $2 ~ /^SYSTEM_LIMIT_/ && 1 == is_impl_dependent {
  define_ident = $2;
  define_value = $3;

  # print the comment
  for (i = 4; i <= NF; i++) {
    if (i != NF) {
      printf("%s ", $i);
    } else {
      print $i;
    }
  }

  printf("%-8s %s\n", "#ifndef", define_ident);
  printf("%-8s %-38s %s\n", $1, $2, $3);
  print "#endif";
  next;
}

# make sure all base APEX types are correctly sized
"typedef" == $1 && $0 ~ /\sAPEX_\w+;\s/ {
  gsub(/\s+unsigned char\s+/, "  uint8_t   ");
  gsub(/\s+unsigned long\s+/, "  uint32_t  ");
  gsub(/\s+long long\s+/, "  int64_t   ");
  gsub(/\s+long\s+/, "  int32_t   ");
}

# always pick `APEX_LONG_INTEGER` (which ist `int64_t`) where there is a choice
{
  gsub("<an APEX integer type>", "APEX_LONG_INTEGER     ");
  print;
}

END {
  print "#include \"private/wasm_apex_proc_alloc.h\""
}

struct IntegerWithPadding {
  unsigned char a;
  long long b;
};

struct FloatWithPadding {
  float a;
  double b;
};

struct IntegerWithArray {
  char a;
  unsigned int b;
  unsigned char c[11];
};

struct MultiDimensionalArray {
  signed char a[7][11];
};

struct NestedStruct {
  struct IntegerWithArray a;
  struct FloatWithPadding b;
};

enum SomeEnum {
  Low,
  High,
  DontCare,
};

struct StructWithEnum {
  char a[3];
  enum SomeEnum b;
};

union SomeUnion {
  int a;
  char b;
  double c;
};

struct StructWithUnion {
  short a;
  union SomeUnion b;
};

struct StructWithPointer {
  unsigned long long a;
  char b;
  struct NestedStruct *c;
};

struct __attribute__((packed)) PackedStruct {
  char a;
  unsigned long long b;
  char c;
  long d[5];
};

typedef struct {
  unsigned short a;
  int b;
} AnonymousStructTypedef;

typedef struct StructThatWillBeTypedefed {
  short a;
  float b;
} StructTypedef;

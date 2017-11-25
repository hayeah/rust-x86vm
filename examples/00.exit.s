.section __TEXT,__text
  .globl start

start:
  mov $0x1, %eax
  push $0x1
  int $0x80

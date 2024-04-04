# JITRepl

A JIT compiler that transforms given mathematical equation into machine code and executes it to
get the result of given equation.

## Requirements

- 64-bit Linux Distro
- x86_64/amd64 processor with SSE support

## Example

```
> (1 + 2) + 3 * 4
```

Above expression makes the JIT compiler generate the following binary code:

```
0000000 4855 e589 8348 10ec 0ff3 0c7f 4824 ec83
0000010 f310 7f0f 240c ba49 0000 0000 0000 4010
0000020 4966 6e0f f3c2 7e0f 49c8 00ba 0000 0000
0000030 0800 6640 0f49 c26e 0ff2 c159 0ff3 0c6f
0000040 4824 c483 f310 7e0f 48c8 ec83 f310 7f0f
0000050 240c ba49 0000 0000 0000 4000 4966 6e0f
0000060 f3c2 7e0f 49c8 00ba 0000 0000 f000 663f
0000070 0f49 c26e 0ff2 c158 0ff3 0c6f 4824 c483
0000080 f210 580f f3c1 6f0f 240c 8348 10c4 8948
0000090 5dec 00c3
0000093
```

Disassembled:

```asm
   0:	55                   	push   %rbp
   1:	48 89 e5             	mov    %rsp,%rbp
   4:	48 83 ec 10          	sub    $0x10,%rsp
   8:	f3 0f 7f 0c 24       	movdqu %xmm1,(%rsp)
   d:	48 83 ec 10          	sub    $0x10,%rsp
  11:	f3 0f 7f 0c 24       	movdqu %xmm1,(%rsp)
  16:	49 ba 00 00 00 00 00 	movabs $0x4010000000000000,%r10
  1d:	00 10 40
  20:	66 49 0f 6e c2       	movq   %r10,%xmm0
  25:	f3 0f 7e c8          	movq   %xmm0,%xmm1
  29:	49 ba 00 00 00 00 00 	movabs $0x4008000000000000,%r10
  30:	00 08 40
  33:	66 49 0f 6e c2       	movq   %r10,%xmm0
  38:	f2 0f 59 c1          	mulsd  %xmm1,%xmm0
  3c:	f3 0f 6f 0c 24       	movdqu (%rsp),%xmm1
  41:	48 83 c4 10          	add    $0x10,%rsp
  45:	f3 0f 7e c8          	movq   %xmm0,%xmm1
  49:	48 83 ec 10          	sub    $0x10,%rsp
  4d:	f3 0f 7f 0c 24       	movdqu %xmm1,(%rsp)
  52:	49 ba 00 00 00 00 00 	movabs $0x4000000000000000,%r10
  59:	00 00 40
  5c:	66 49 0f 6e c2       	movq   %r10,%xmm0
  61:	f3 0f 7e c8          	movq   %xmm0,%xmm1
  65:	49 ba 00 00 00 00 00 	movabs $0x3ff0000000000000,%r10
  6c:	00 f0 3f
  6f:	66 49 0f 6e c2       	movq   %r10,%xmm0
  74:	f2 0f 58 c1          	addsd  %xmm1,%xmm0
  78:	f3 0f 6f 0c 24       	movdqu (%rsp),%xmm1
  7d:	48 83 c4 10          	add    $0x10,%rsp
  81:	f2 0f 58 c1          	addsd  %xmm1,%xmm0
  85:	f3 0f 6f 0c 24       	movdqu (%rsp),%xmm1
  8a:	48 83 c4 10          	add    $0x10,%rsp
  8e:	48 89 ec             	mov    %rbp,%rsp
  91:	5d                   	pop    %rbp
  92:	c3                   	ret
```

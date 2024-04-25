; ModuleID = 'examples/ll/brainfuck.c'
source_filename = "examples/ll/brainfuck.c"
target datalayout = "e-m:o-i64:64-i128:128-n32:64-S128"
target triple = "arm64-apple-macosx14.0.0"

@.str = private unnamed_addr constant [3 x i8] c"%c\00", align 1
@.str.1 = private unnamed_addr constant [2 x i8] c"\0A\00", align 1
@__stderrp = external global ptr, align 8
@.str.2 = private unnamed_addr constant [18 x i8] c"invalid arguments\00", align 1

; Function Attrs: noinline nounwind optnone ssp uwtable(sync)
define void @brainfuck(ptr noundef %0, ptr noundef %1) #0 {
  %3 = alloca ptr, align 8
  %4 = alloca ptr, align 8
  %5 = alloca i32, align 4
  %6 = alloca i8, align 1
  %7 = alloca [1001 x i8], align 1
  %8 = alloca ptr, align 8
  store ptr %0, ptr %3, align 8
  store ptr %1, ptr %4, align 8
  call void @llvm.memset.p0.i64(ptr align 1 %7, i8 0, i64 1001, i1 false)
  %9 = getelementptr inbounds [1001 x i8], ptr %7, i64 0, i64 500
  store ptr %9, ptr %8, align 8
  br label %10

10:                                               ; preds = %108, %2
  %11 = load ptr, ptr %3, align 8
  %12 = getelementptr inbounds i8, ptr %11, i32 1
  store ptr %12, ptr %3, align 8
  %13 = load i8, ptr %11, align 1
  store i8 %13, ptr %6, align 1
  %14 = icmp ne i8 %13, 0
  br i1 %14, label %15, label %109

15:                                               ; preds = %10
  %16 = load i8, ptr %6, align 1
  %17 = sext i8 %16 to i32
  switch i32 %17, label %108 [
    i32 62, label %18
    i32 60, label %21
    i32 43, label %24
    i32 45, label %28
    i32 46, label %32
    i32 44, label %37
    i32 91, label %42
    i32 93, label %73
  ]

18:                                               ; preds = %15
  %19 = load ptr, ptr %8, align 8
  %20 = getelementptr inbounds i8, ptr %19, i32 1
  store ptr %20, ptr %8, align 8
  br label %108

21:                                               ; preds = %15
  %22 = load ptr, ptr %8, align 8
  %23 = getelementptr inbounds i8, ptr %22, i32 -1
  store ptr %23, ptr %8, align 8
  br label %108

24:                                               ; preds = %15
  %25 = load ptr, ptr %8, align 8
  %26 = load i8, ptr %25, align 1
  %27 = add i8 %26, 1
  store i8 %27, ptr %25, align 1
  br label %108

28:                                               ; preds = %15
  %29 = load ptr, ptr %8, align 8
  %30 = load i8, ptr %29, align 1
  %31 = add i8 %30, -1
  store i8 %31, ptr %29, align 1
  br label %108

32:                                               ; preds = %15
  %33 = load ptr, ptr %8, align 8
  %34 = load i8, ptr %33, align 1
  %35 = sext i8 %34 to i32
  %36 = call i32 (ptr, ...) @printf(ptr noundef @.str, i32 noundef %35)
  br label %108

37:                                               ; preds = %15
  %38 = load ptr, ptr %4, align 8
  %39 = getelementptr inbounds i8, ptr %38, i32 1
  store ptr %39, ptr %4, align 8
  %40 = load i8, ptr %38, align 1
  %41 = load ptr, ptr %8, align 8
  store i8 %40, ptr %41, align 1
  br label %108

42:                                               ; preds = %15
  %43 = load ptr, ptr %8, align 8
  %44 = load i8, ptr %43, align 1
  %45 = icmp ne i8 %44, 0
  br i1 %45, label %72, label %46

46:                                               ; preds = %42
  store i32 1, ptr %5, align 4
  br label %47

47:                                               ; preds = %68, %46
  %48 = load i32, ptr %5, align 4
  %49 = icmp ne i32 %48, 0
  br i1 %49, label %50, label %71

50:                                               ; preds = %47
  %51 = load ptr, ptr %3, align 8
  %52 = load i8, ptr %51, align 1
  %53 = sext i8 %52 to i32
  %54 = icmp eq i32 %53, 91
  br i1 %54, label %55, label %58

55:                                               ; preds = %50
  %56 = load i32, ptr %5, align 4
  %57 = add nsw i32 %56, 1
  store i32 %57, ptr %5, align 4
  br label %67

58:                                               ; preds = %50
  %59 = load ptr, ptr %3, align 8
  %60 = load i8, ptr %59, align 1
  %61 = sext i8 %60 to i32
  %62 = icmp eq i32 %61, 93
  br i1 %62, label %63, label %66

63:                                               ; preds = %58
  %64 = load i32, ptr %5, align 4
  %65 = add nsw i32 %64, -1
  store i32 %65, ptr %5, align 4
  br label %66

66:                                               ; preds = %63, %58
  br label %67

67:                                               ; preds = %66, %55
  br label %68

68:                                               ; preds = %67
  %69 = load ptr, ptr %3, align 8
  %70 = getelementptr inbounds i8, ptr %69, i32 1
  store ptr %70, ptr %3, align 8
  br label %47, !llvm.loop !5

71:                                               ; preds = %47
  br label %72

72:                                               ; preds = %71, %42
  br label %108

73:                                               ; preds = %15
  %74 = load ptr, ptr %8, align 8
  %75 = load i8, ptr %74, align 1
  %76 = icmp ne i8 %75, 0
  br i1 %76, label %77, label %107

77:                                               ; preds = %73
  %78 = load ptr, ptr %3, align 8
  %79 = getelementptr inbounds i8, ptr %78, i64 -2
  store ptr %79, ptr %3, align 8
  store i32 1, ptr %5, align 4
  br label %80

80:                                               ; preds = %101, %77
  %81 = load i32, ptr %5, align 4
  %82 = icmp ne i32 %81, 0
  br i1 %82, label %83, label %104

83:                                               ; preds = %80
  %84 = load ptr, ptr %3, align 8
  %85 = load i8, ptr %84, align 1
  %86 = sext i8 %85 to i32
  %87 = icmp eq i32 %86, 93
  br i1 %87, label %88, label %91

88:                                               ; preds = %83
  %89 = load i32, ptr %5, align 4
  %90 = add nsw i32 %89, 1
  store i32 %90, ptr %5, align 4
  br label %100

91:                                               ; preds = %83
  %92 = load ptr, ptr %3, align 8
  %93 = load i8, ptr %92, align 1
  %94 = sext i8 %93 to i32
  %95 = icmp eq i32 %94, 91
  br i1 %95, label %96, label %99

96:                                               ; preds = %91
  %97 = load i32, ptr %5, align 4
  %98 = add nsw i32 %97, -1
  store i32 %98, ptr %5, align 4
  br label %99

99:                                               ; preds = %96, %91
  br label %100

100:                                              ; preds = %99, %88
  br label %101

101:                                              ; preds = %100
  %102 = load ptr, ptr %3, align 8
  %103 = getelementptr inbounds i8, ptr %102, i32 -1
  store ptr %103, ptr %3, align 8
  br label %80, !llvm.loop !7

104:                                              ; preds = %80
  %105 = load ptr, ptr %3, align 8
  %106 = getelementptr inbounds i8, ptr %105, i32 1
  store ptr %106, ptr %3, align 8
  br label %107

107:                                              ; preds = %104, %73
  br label %108

108:                                              ; preds = %15, %107, %72, %37, %32, %28, %24, %21, %18
  br label %10, !llvm.loop !8

109:                                              ; preds = %10
  %110 = call i32 (ptr, ...) @printf(ptr noundef @.str.1)
  ret void
}

; Function Attrs: nocallback nofree nounwind willreturn memory(argmem: write)
declare void @llvm.memset.p0.i64(ptr nocapture writeonly, i8, i64, i1 immarg) #1

declare i32 @printf(ptr noundef, ...) #2

; Function Attrs: noinline nounwind optnone ssp uwtable(sync)
define i32 @main(i32 noundef %0, ptr noundef %1) #0 {
  %3 = alloca i32, align 4
  %4 = alloca i32, align 4
  %5 = alloca ptr, align 8
  store i32 0, ptr %3, align 4
  store i32 %0, ptr %4, align 4
  store ptr %1, ptr %5, align 8
  %6 = load i32, ptr %4, align 4
  %7 = icmp ne i32 %6, 3
  br i1 %7, label %8, label %11

8:                                                ; preds = %2
  %9 = load ptr, ptr @__stderrp, align 8
  %10 = call i32 (ptr, ptr, ...) @fprintf(ptr noundef %9, ptr noundef @.str.2)
  store i32 1, ptr %3, align 4
  br label %18

11:                                               ; preds = %2
  %12 = load ptr, ptr %5, align 8
  %13 = getelementptr inbounds ptr, ptr %12, i64 1
  %14 = load ptr, ptr %13, align 8
  %15 = load ptr, ptr %5, align 8
  %16 = getelementptr inbounds ptr, ptr %15, i64 2
  %17 = load ptr, ptr %16, align 8
  call void @brainfuck(ptr noundef %14, ptr noundef %17)
  br label %18

18:                                               ; preds = %11, %8
  %19 = load i32, ptr %3, align 4
  ret i32 %19
}

declare i32 @fprintf(ptr noundef, ptr noundef, ...) #2

attributes #0 = { noinline nounwind optnone ssp uwtable(sync) "frame-pointer"="non-leaf" "no-trapping-math"="true" "stack-protector-buffer-size"="8" "target-cpu"="apple-m1" "target-features"="+aes,+crc,+dotprod,+fp-armv8,+fp16fml,+fullfp16,+lse,+neon,+ras,+rcpc,+rdm,+sha2,+sha3,+v8.1a,+v8.2a,+v8.3a,+v8.4a,+v8.5a,+v8a,+zcm,+zcz" }
attributes #1 = { nocallback nofree nounwind willreturn memory(argmem: write) }
attributes #2 = { "frame-pointer"="non-leaf" "no-trapping-math"="true" "stack-protector-buffer-size"="8" "target-cpu"="apple-m1" "target-features"="+aes,+crc,+dotprod,+fp-armv8,+fp16fml,+fullfp16,+lse,+neon,+ras,+rcpc,+rdm,+sha2,+sha3,+v8.1a,+v8.2a,+v8.3a,+v8.4a,+v8.5a,+v8a,+zcm,+zcz" }

!llvm.module.flags = !{!0, !1, !2, !3}
!llvm.ident = !{!4}

!0 = !{i32 1, !"wchar_size", i32 4}
!1 = !{i32 8, !"PIC Level", i32 2}
!2 = !{i32 7, !"uwtable", i32 1}
!3 = !{i32 7, !"frame-pointer", i32 1}
!4 = !{!"Homebrew clang version 17.0.6"}
!5 = distinct !{!5, !6}
!6 = !{!"llvm.loop.mustprogress"}
!7 = distinct !{!7, !6}
!8 = distinct !{!8, !6}

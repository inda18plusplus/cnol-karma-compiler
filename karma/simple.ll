; ModuleID = 'karma'
source_filename = "karma"

@format = global [4 x i8] c"%ld\00"
@stack = global i64* null
@stack_length = global i64 0
@stack_capacity = global i64 0
@deque = global i64* null
@deque_back = global i64 0
@deque_length = global i64 0
@deque_capacity = global i64 0
@next_section_0 = global i64 0
@next_section_1 = global i64 0
@next_section_2 = global i64 0
@next_section_3 = global i64 0

declare i8* @malloc(i32)

declare void @free(i8*)

declare void @memcpy(i8*, i8*, i64)

declare void @exit(i32)

declare i32 @getchar()

declare i32 @putchar(i32)

declare i32 @printf(i8*, ...)

define void @puti64(i64 %value) {
entry:
  %0 = call i32 (i8*, ...) @printf(i8* getelementptr inbounds ([4 x i8], [4 x i8]* @format, i32 0, i32 0), i64 %value)
  ret void
}

define void @stack_resize(i64 %new_size) {
entry:
  %0 = load i64, i64* @stack_capacity
  %1 = trunc i64 %new_size to i32
  %2 = mul i32 %1, 8
  %3 = mul i64 %0, 8
  %4 = call i8* @malloc(i32 %2)
  %5 = load i64*, i64** @stack
  %6 = bitcast i64* %5 to i8*
  call void @memcpy(i8* %4, i8* %6, i64 %3)
  call void @free(i8* %6)
  %7 = bitcast i8* %4 to i64*
  store i64* %7, i64** @stack
  %8 = sext i32 %1 to i64
  store i64 %8, i64* @stack_capacity
  ret void
}

define void @push(i64 %value) {
entry:
  %0 = load i64, i64* @stack_length
  %1 = load i64, i64* @stack_capacity
  %2 = add i64 %0, 1
  store i64 %2, i64* @stack_length
  %3 = icmp sgt i64 %2, %1
  br i1 %3, label %grow, label %write

grow:                                             ; preds = %entry
  %4 = load i64, i64* @stack_capacity
  %5 = mul i64 %4, 2
  call void @stack_resize(i64 %5)
  br label %write

write:                                            ; preds = %grow, %entry
  %6 = load i64, i64* @stack_length
  %7 = sub i64 %6, 1
  %8 = load i64*, i64** @stack
  %9 = getelementptr i64, i64* %8, i64 %7
  store i64 %value, i64* %9
  ret void
}

define i64 @pop() {
entry:
  %0 = load i64, i64* @stack_length
  %1 = sub i64 %0, 1
  store i64 %1, i64* @stack_length
  %2 = icmp sgt i64 0, %1
  br i1 %2, label %fail, label %read

fail:                                             ; preds = %entry
  call void @exit(i32 14)
  ret i64 -1

read:                                             ; preds = %entry
  %3 = load i64, i64* @stack_length
  %4 = load i64*, i64** @stack
  %5 = getelementptr i64, i64* %4, i64 %3
  %6 = load i64, i64* %5
  ret i64 %6
}

define void @insert_front(i64 %value) {
entry:
  %0 = load i64, i64* @deque_length
  %1 = load i64, i64* @deque_capacity
  %2 = icmp sge i64 %0, %1
  br i1 %2, label %grow, label %exit

exit:                                             ; preds = %entry, %grow
  %3 = load i64, i64* @deque_back
  %4 = load i64, i64* @deque_length
  %5 = load i64, i64* @deque_capacity
  %6 = add i64 %3, %4
  %7 = srem i64 %6, %5
  %8 = load i64*, i64** @deque
  %9 = getelementptr i64, i64* %8, i64 %7
  store i64 %value, i64* %9
  %10 = load i64, i64* @deque_length
  %11 = add i64 %10, 1
  store i64 %11, i64* @deque_length
  ret void

grow:                                             ; preds = %entry
  %12 = load i64, i64* @deque_capacity
  %13 = mul i64 %12, 2
  call void @deque_resize(i64 %13)
  br label %exit
}

define void @insert_back(i64 %value) {
entry:
  %0 = load i64, i64* @deque_length
  %1 = load i64, i64* @deque_capacity
  %2 = icmp sge i64 %0, %1
  br i1 %2, label %grow, label %exit

exit:                                             ; preds = %entry, %grow
  %3 = load i64, i64* @deque_length
  %4 = add i64 %3, 1
  store i64 %4, i64* @deque_length
  %5 = load i64, i64* @deque_back
  %6 = load i64, i64* @deque_capacity
  %7 = add i64 %5, -1
  %8 = add i64 %7, %6
  %9 = srem i64 %8, %6
  store i64 %9, i64* @deque_back
  %10 = load i64*, i64** @deque
  %11 = getelementptr i64, i64* %10, i64 %9
  store i64 %value, i64* %11
  ret void

grow:                                             ; preds = %entry
  %12 = load i64, i64* @deque_capacity
  %13 = mul i64 %12, 2
  call void @deque_resize(i64 %13)
  br label %exit
}

define i64 @remove_front() {
entry:
  %0 = load i64, i64* @deque_length
  %1 = icmp sgt i64 0, %0
  br i1 %1, label %fail, label %exit

exit:                                             ; preds = %entry
  %2 = load i64, i64* @deque_length
  %3 = add i64 %2, -1
  store i64 %3, i64* @deque_length
  %4 = load i64, i64* @deque_back
  %5 = load i64, i64* @deque_length
  %6 = load i64, i64* @deque_capacity
  %7 = add i64 %4, %5
  %8 = srem i64 %7, %6
  %9 = load i64*, i64** @deque
  %10 = getelementptr i64, i64* %9, i64 %8
  %11 = load i64, i64* %10
  ret i64 %11

fail:                                             ; preds = %entry
  call void @exit(i32 13)
  ret i64 -1
}

define i64 @remove_back() {
entry:
  %0 = load i64, i64* @deque_length
  %1 = icmp sgt i64 0, %0
  br i1 %1, label %fail, label %exit

exit:                                             ; preds = %entry
  %2 = load i64, i64* @deque_back
  %3 = load i64*, i64** @deque
  %4 = getelementptr i64, i64* %3, i64 %2
  %5 = load i64, i64* %4
  %6 = load i64, i64* @deque_length
  %7 = add i64 %6, -1
  store i64 %7, i64* @deque_length
  %8 = load i64, i64* @deque_back
  %9 = load i64, i64* @deque_capacity
  %10 = add i64 %8, 1
  %11 = add i64 %10, %9
  %12 = srem i64 %11, %9
  store i64 %12, i64* @deque_back
  ret i64 %5

fail:                                             ; preds = %entry
  call void @exit(i32 13)
  ret i64 -1
}

define void @deque_resize(i64 %new_size) {
entry:
  %0 = mul i64 %new_size, 8
  %1 = trunc i64 %0 to i32
  %2 = call i8* @malloc(i32 %1)
  %3 = load i64, i64* @deque_capacity
  %4 = icmp eq i64 %3, 0
  br i1 %4, label %exit, label %find_layout

find_layout:                                      ; preds = %entry
  %5 = load i64, i64* @deque_back
  %6 = load i64, i64* @deque_back
  %7 = load i64, i64* @deque_length
  %8 = load i64, i64* @deque_capacity
  %9 = add i64 %6, %7
  %10 = srem i64 %9, %8
  %11 = icmp sge i64 %5, %10
  br i1 %11, label %wrapping, label %linear

wrapping:                                         ; preds = %find_layout
  %12 = load i64, i64* @deque_capacity
  %13 = load i64, i64* @deque_back
  %14 = load i64, i64* @deque_back
  %15 = load i64, i64* @deque_length
  %16 = load i64, i64* @deque_capacity
  %17 = add i64 %14, %15
  %18 = srem i64 %17, %16
  %19 = load i64*, i64** @deque
  %20 = bitcast i64* %19 to i8*
  %21 = mul i64 %18, 8
  call void @memcpy(i8* %2, i8* %20, i64 %21)
  %22 = sub i64 %12, %13
  %23 = mul i64 %22, 8
  %24 = sub i64 %0, %23
  %25 = getelementptr i8, i8* %2, i64 %24
  %26 = getelementptr i64, i64* %19, i64 %13
  %27 = bitcast i64* %26 to i8*
  call void @memcpy(i8* %25, i8* %27, i64 %23)
  %28 = sub i64 %new_size, %22
  store i64 %28, i64* @deque_back
  br label %exit

linear:                                           ; preds = %find_layout
  %29 = load i64, i64* @deque_capacity
  %30 = load i64*, i64** @deque
  %31 = bitcast i64* %30 to i8*
  %32 = mul i64 %29, 8
  call void @memcpy(i8* %2, i8* %31, i64 %32)
  br label %exit

exit:                                             ; preds = %wrapping, %linear, %entry
  %33 = load i64*, i64** @deque
  %34 = bitcast i64* %33 to i8*
  call void @free(i8* %34)
  %35 = bitcast i8* %2 to i64*
  store i64* %35, i64** @deque
  store i64 %new_size, i64* @deque_capacity
  ret void
}

define i32 @main() {
init_stack:
  call void @stack_resize(i64 16)
  br label %init_deque

init_deque:                                       ; preds = %init_stack
  call void @deque_resize(i64 16)
  br label %entry

entry:                                            ; preds = %init_deque
  store i64 0, i64* @next_section_1
  br label %jump_table_1

exit:                                             ; preds = %section_3_0, %section_2_0, %section_1_1, %section_0_0
  ret i32 0

panic:                                            ; preds = %jump_table_3, %jump_table_2, %jump_table_1, %jump_table_0
  ret i32 1

section_0_0:                                      ; preds = %jump_table_0
  br label %exit

jump_table_0:                                     ; No predecessors!
  %0 = load i64, i64* @next_section_0
  switch i64 %0, label %panic [
    i64 0, label %section_0_0
  ]

section_1_0:                                      ; preds = %jump_table_1
  call void @push(i64 10000)
  store i64 1, i64* @next_section_1
  store i64 0, i64* @next_section_2
  br label %jump_table_2

section_1_1:                                      ; preds = %jump_table_1
  br label %exit

jump_table_1:                                     ; preds = %entry
  %1 = load i64, i64* @next_section_1
  switch i64 %1, label %panic [
    i64 0, label %section_1_0
    i64 1, label %section_1_1
  ]

section_2_0:                                      ; preds = %jump_table_2
  %2 = call i64 @pop()
  call void @puti64(i64 %2)
  br label %exit

jump_table_2:                                     ; preds = %section_1_0
  %3 = load i64, i64* @next_section_2
  switch i64 %3, label %panic [
    i64 0, label %section_2_0
  ]

section_3_0:                                      ; preds = %jump_table_3
  br label %exit

jump_table_3:                                     ; No predecessors!
  %4 = load i64, i64* @next_section_3
  switch i64 %4, label %panic [
    i64 0, label %section_3_0
  ]
}

# initial

```
define noundef i32 @main(i32 noundef %0, ptr nocapture noundef readnone %1) local_unnamed_addr #0 {
def use phi in out
 %3  %0                 %3 = icmp sgt i32 %0, 0
     %3                 br i1 %3, label %5, label %4
    
                      4:                                                ; preds = %5, %2
                        ret i32 0
   
                      5:                                                ; preds = %2, %5
                        %6 = phi i32 [ %8, %5 ], [ 0, %2 ]
 %7  %6                 %7 = tail call i32 (ptr, ...) @printf(ptr noundef nonnull dereferenceable(1) @.str, i32 noundef %6)
 %8  %6                 %8 = add nuw nsw i32 %6, 1
 %9  %0,%8              %9 = icmp eq i32 %8, %0
     %9  %8             br i1 %9, label %4, label %5, !llvm.loop !5
}
```

# step 1 (in)

```
define noundef i32 @main(i32 noundef %0, ptr nocapture noundef readnone %1) local_unnamed_addr #0 {
def    use phi    in out
 %3     %0        %0       %3 = icmp sgt i32 %0, 0
        %3        %3       br i1 %3, label %5, label %4
    
                         4:                                                ; preds = %5, %2
                           ret i32 0
   
                         5:                                                ; preds = %2, %5
                           %6 = phi i32 [ %8, %5 ], [ 0, %2 ]
 %7     %6        %6       %7 = tail call i32 (ptr, ...) @printf(ptr noundef nonnull dereferenceable(1) @.str, i32 noundef %6)
 %8     %6        %6       %8 = add nuw nsw i32 %6, 1
 %9  %0,%8     %0,%8       %9 = icmp eq i32 %8, %0
        %9  %8    %9       br i1 %9, label %4, label %5, !llvm.loop !5
}
```

# step 1 (out)

```
define noundef i32 @main(i32 noundef %0, ptr nocapture noundef readnone %1) local_unnamed_addr #0 {
def    use phi    in    out
 %3     %0        %0     %3   %3 = icmp sgt i32 %0, 0
        %3        %3          br i1 %3, label %5, label %4

                            4:                                                ; preds = %5, %2
                              ret i32 0

                            5:                                                ; preds = %2, %5
                         %6   %6 = phi i32 [ %8, %5 ], [ 0, %2 ]
 %7     %6        %6     %6   %7 = tail call i32 (ptr, ...) @printf(ptr noundef nonnull dereferenceable(1) @.str, i32 noundef %6)
 %8     %6        %6  %0,%8   %8 = add nuw nsw i32 %6, 1
 %9  %0,%8     %0,%8     %9   %9 = icmp eq i32 %8, %0
        %9  %8    %9     %8   br i1 %9, label %4, label %5, !llvm.loop !5
}
```

# step 2 (in)

```
define noundef i32 @main(i32 noundef %0, ptr nocapture noundef readnone %1) local_unnamed_addr #0 {
def    use phi    in    out
 %3     %0        %0     %3   %3 = icmp sgt i32 %0, 0
        %3        %3          br i1 %3, label %5, label %4

                            4:                                                ; preds = %5, %2
                              ret i32 0

                            5:                                                ; preds = %2, %5
                         %6   %6 = phi i32 [ %8, %5 ], [ 0, %2 ]
 %7     %6        %6     %6   %7 = tail call i32 (ptr, ...) @printf(ptr noundef nonnull dereferenceable(1) @.str, i32 noundef %6)
 %8     %6     %0,%6  %0,%8   %8 = add nuw nsw i32 %6, 1
 %9  %0,%8     %0,%8     %9   %9 = icmp eq i32 %8, %0
        %9  %8 %8,%9     %8   br i1 %9, label %4, label %5, !llvm.loop !5
}
```

# step 2 (out)

```
define noundef i32 @main(i32 noundef %0, ptr nocapture noundef readnone %1) local_unnamed_addr #0 {
def    use phi    in    out
 %3     %0        %0     %3   %3 = icmp sgt i32 %0, 0
        %3        %3          br i1 %3, label %5, label %4

                            4:                                                ; preds = %5, %2
                              ret i32 0

                            5:                                                ; preds = %2, %5
                         %6   %6 = phi i32 [ %8, %5 ], [ 0, %2 ]
 %7     %6        %6  %0,%6   %7 = tail call i32 (ptr, ...) @printf(ptr noundef nonnull dereferenceable(1) @.str, i32 noundef %6)
 %8     %6     %0,%6  %0,%8   %8 = add nuw nsw i32 %6, 1
 %9  %0,%8     %0,%8  %8,%9   %9 = icmp eq i32 %8, %0
        %9  %8 %8,%9     %8   br i1 %9, label %4, label %5, !llvm.loop !5
}
```

# step 3 (in)

```
define noundef i32 @main(i32 noundef %0, ptr nocapture noundef readnone %1) local_unnamed_addr #0 {
def    use phi    in    out
 %3     %0        %0     %3   %3 = icmp sgt i32 %0, 0
        %3        %3          br i1 %3, label %5, label %4

                            4:                                                ; preds = %5, %2
                              ret i32 0

                            5:                                                ; preds = %2, %5
                         %6   %6 = phi i32 [ %8, %5 ], [ 0, %2 ]
 %7     %6     %0,%6  %0,%6   %7 = tail call i32 (ptr, ...) @printf(ptr noundef nonnull dereferenceable(1) @.str, i32 noundef %6)
 %8     %6     %0,%6  %0,%8   %8 = add nuw nsw i32 %6, 1
 %9  %0,%8     %0,%8  %8,%9   %9 = icmp eq i32 %8, %0
        %9  %8 %8,%9     %8   br i1 %9, label %4, label %5, !llvm.loop !5
}
```

# step 3 (out)

```
define noundef i32 @main(i32 noundef %0, ptr nocapture noundef readnone %1) local_unnamed_addr #0 {
def    use phi    in    out
 %3     %0        %0     %3   %3 = icmp sgt i32 %0, 0
        %3        %3          br i1 %3, label %5, label %4

                            4:                                                ; preds = %5, %2
                              ret i32 0

                            5:                                                ; preds = %2, %5
                      %0,%6   %6 = phi i32 [ %8, %5 ], [ 0, %2 ]
 %7     %6     %0,%6  %0,%6   %7 = tail call i32 (ptr, ...) @printf(ptr noundef nonnull dereferenceable(1) @.str, i32 noundef %6)
 %8     %6     %0,%6  %0,%8   %8 = add nuw nsw i32 %6, 1
 %9  %0,%8     %0,%8  %8,%9   %9 = icmp eq i32 %8, %0
        %9  %8 %8,%9     %8   br i1 %9, label %4, label %5, !llvm.loop !5
}
```

# step 4 (in)

```
define noundef i32 @main(i32 noundef %0, ptr nocapture noundef readnone %1) local_unnamed_addr #0 {
def    use phi    in    out
 %3     %0        %0     %3   %3 = icmp sgt i32 %0, 0
        %3        %3          br i1 %3, label %5, label %4

                            4:                                                ; preds = %5, %2
                              ret i32 0

                            5:                                                ; preds = %2, %5
                  %0  %0,%6   %6 = phi i32 [ %8, %5 ], [ 0, %2 ]
 %7     %6     %0,%6  %0,%6   %7 = tail call i32 (ptr, ...) @printf(ptr noundef nonnull dereferenceable(1) @.str, i32 noundef %6)
 %8     %6     %0,%6  %0,%8   %8 = add nuw nsw i32 %6, 1
 %9  %0,%8     %0,%8  %8,%9   %9 = icmp eq i32 %8, %0
        %9  %8 %8,%9     %8   br i1 %9, label %4, label %5, !llvm.loop !5
}
```
```

# step 4 (out)

```
define noundef i32 @main(i32 noundef %0, ptr nocapture noundef readnone %1) local_unnamed_addr #0 {
def    use phi    in    out
 %3     %0        %0     %3   %3 = icmp sgt i32 %0, 0
        %3        %3          br i1 %3, label %5, label %4

                            4:                                                ; preds = %5, %2
                              ret i32 0

                            5:                                                ; preds = %2, %5
                  %0  %0,%6   %6 = phi i32 [ %8, %5 ], [ 0, %2 ]
 %7     %6     %0,%6  %0,%6   %7 = tail call i32 (ptr, ...) @printf(ptr noundef nonnull dereferenceable(1) @.str, i32 noundef %6)
 %8     %6     %0,%6  %0,%8   %8 = add nuw nsw i32 %6, 1
 %9  %0,%8     %0,%8  %8,%9   %9 = icmp eq i32 %8, %0
        %9  %8 %8,%9     %8   br i1 %9, label %4, label %5, !llvm.loop !5
}
```

# step 5 (in)

```
define noundef i32 @main(i32 noundef %0, ptr nocapture noundef readnone %1) local_unnamed_addr #0 {
def    use phi    in    out
 %3     %0        %0     %3   %3 = icmp sgt i32 %0, 0
        %3        %3          br i1 %3, label %5, label %4

                            4:                                                ; preds = %5, %2
                              ret i32 0

                            5:                                                ; preds = %2, %5
                  %0  %0,%6   %6 = phi i32 [ %8, %5 ], [ 0, %2 ]
 %7     %6     %0,%6  %0,%6   %7 = tail call i32 (ptr, ...) @printf(ptr noundef nonnull dereferenceable(1) @.str, i32 noundef %6)
 %8     %6     %0,%6  %0,%8   %8 = add nuw nsw i32 %6, 1
 %9  %0,%8     %0,%8  %8,%9   %9 = icmp eq i32 %8, %0
        %9  %8 %8,%9     %8   br i1 %9, label %4, label %5, !llvm.loop !5
}
```
```

# step 5 (out)

```
define noundef i32 @main(i32 noundef %0, ptr nocapture noundef readnone %1) local_unnamed_addr #0 {
def    use phi    in    out
 %3     %0        %0     %3   %3 = icmp sgt i32 %0, 0
        %3        %3     %0   br i1 %3, label %5, label %4

                            4:                                                ; preds = %5, %2
                              ret i32 0

                            5:                                                ; preds = %2, %5
                  %0  %0,%6   %6 = phi i32 [ %8, %5 ], [ 0, %2 ]
 %7     %6     %0,%6  %0,%6   %7 = tail call i32 (ptr, ...) @printf(ptr noundef nonnull dereferenceable(1) @.str, i32 noundef %6)
 %8     %6     %0,%6  %0,%8   %8 = add nuw nsw i32 %6, 1
 %9  %0,%8     %0,%8  %8,%9   %9 = icmp eq i32 %8, %0
        %9  %8 %8,%9     %8   br i1 %9, label %4, label %5, !llvm.loop !5
}
```

# step 6 (in)

```
define noundef i32 @main(i32 noundef %0, ptr nocapture noundef readnone %1) local_unnamed_addr #0 {
def    use phi    in    out
 %3     %0        %0     %3   %3 = icmp sgt i32 %0, 0
        %3     %0,%3     %0   br i1 %3, label %5, label %4

                            4:                                                ; preds = %5, %2
                              ret i32 0

                            5:                                                ; preds = %2, %5
                  %0  %0,%6   %6 = phi i32 [ %8, %5 ], [ 0, %2 ]
 %7     %6     %0,%6  %0,%6   %7 = tail call i32 (ptr, ...) @printf(ptr noundef nonnull dereferenceable(1) @.str, i32 noundef %6)
 %8     %6     %0,%6  %0,%8   %8 = add nuw nsw i32 %6, 1
 %9  %0,%8     %0,%8  %8,%9   %9 = icmp eq i32 %8, %0
        %9  %8 %8,%9     %8   br i1 %9, label %4, label %5, !llvm.loop !5
}
```
```

# step 6 (out)

```
define noundef i32 @main(i32 noundef %0, ptr nocapture noundef readnone %1) local_unnamed_addr #0 {
def    use phi    in    out
 %3     %0        %0  %0,%3   %3 = icmp sgt i32 %0, 0
        %3     %0,%3     %0   br i1 %3, label %5, label %4

                            4:                                                ; preds = %5, %2
                              ret i32 0

                            5:                                                ; preds = %2, %5
                  %0  %0,%6   %6 = phi i32 [ %8, %5 ], [ 0, %2 ]
 %7     %6     %0,%6  %0,%6   %7 = tail call i32 (ptr, ...) @printf(ptr noundef nonnull dereferenceable(1) @.str, i32 noundef %6)
 %8     %6     %0,%6  %0,%8   %8 = add nuw nsw i32 %6, 1
 %9  %0,%8     %0,%8  %8,%9   %9 = icmp eq i32 %8, %0                       <<-,
        %9  %8 %8,%9     %8   br i1 %9, label %4, label %5, !llvm.loop !5   <<---both of these are wrong, should account for %0!!
}
```

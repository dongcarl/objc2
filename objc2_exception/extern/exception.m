/// This is always available when building Objective-C.
///
/// See <https://clang.llvm.org/docs/AutomaticReferenceCounting.html#arc-runtime-objc-retain>.
id objc_retain(id value);

// We return `unsigned char`, since it is guaranteed to be an `u8` on all platforms
unsigned char RustObjCExceptionTryCatch(void (*f)(void *), void *context, id *error) {
    @try {
        f(context);
        if (error) {
            *error = (id)0; // nil
        }
        return 0;
    } @catch (id exception) {
        if (error) {
            *error = objc_retain(exception);
        }
        return 1;
    }
}
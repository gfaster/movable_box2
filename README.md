## Movable Box

This is just a little proof-of-concept as to make a movable box type for use in
a compacting heap in Rust. The idea is loosely based off of [Brooks
2004][brooks], but adapted a little bit for simplicity in Rust.

Basically, Every allocation has a forwarding header that needs to be checked
whenever `Deref` is called. If the allocation hasn't been moved, execution can
continue as normal. If the allocation has been moved, the pointer must be
updated to the new location and the original block has to be deallocated.

Critically, even when the original allocation is forwarded, the data itself
never has a reference taken to it. This means that this system can be extended
with another thread updating allocations without worrying about violating
`&mut` aliasing rules.

By the same token, this sytem can be extended with other allocation metadata,
whether it be for access frequency or reference counting.


[brooks]: https://www.steveblackburn.org/pubs/papers/wb-ismm-2004.pdf

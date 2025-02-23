# BLOB
A `BLOB`, also know as Binary Large Objects, is a way to generalize and store any kind of binary data, regardless of it's size and underlying encoding or type.

The motivation of this crate is to achieve something akin to `std::any::Any` without dynamic dispatch. This id done via complete type erasure of the stored values, and relegating the programmer to manage separating each type of data into it's own type errased `BLOB`.

# TODO
### v0.1
- [x] Generic safe BLOB.

### v0.2
- [x] Add ZST support.

### v0.3
- [ ] Generic safe Vec like BLOB with basic swap remove.

### v0.4
- [ ] VecBLOB with Vec like interface.
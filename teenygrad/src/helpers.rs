/*
 * Copyright (c) 2023 SpinorML
 *
 * Permission is hereby granted, free of charge, to any person obtaining a copy
 * of this software and associated documentation files (the "Software"), to deal
 * in the Software without restriction, including without limitation the rights
 * to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
 * copies of the Software, and to permit persons to whom the Software is
 * furnished to do so, subject to the following conditions:
 *
 * The above copyright notice and this permission notice shall be included in all
 * copies or substantial portions of the Software.
 *
 * THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
 * IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
 * FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
 * AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
 * LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
 * OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
 * SOFTWARE.
 */

pub type ShapeType = Vec<usize>;

pub fn is_osx() {
    // OSX = platform.system() == "Darwin"
    todo!("Not yet implemented")
}

pub fn dedup(_x: &str) {
    // def dedup(x): return list(dict.fromkeys(x))   # retains list order
    todo!("Not yet implemented")
}

pub fn argfix(_x: &str) {
    // def argfix(*x):
    //   if x[0].__class__ in {tuple, list}:
    //     try: return tuple(x[0])
    //     except IndexError: return tuple()
    //   return tuple(x)
    todo!("Not yet implemented")
}
pub fn argsort(_x: &str) {
    // def argsort(x): return type(x)(sorted(range(len(x)), key=x.__getitem__)) # https://stackoverflow.com/questions/3382352/equivalent-of-numpy-argsort-in-basic-python
    todo!("Not yet implemented")
}

pub fn all_same(_items: &str) {
    // def all_same(items): return all(x == items[0] for x in items)
    todo!("Not yet implemented")
}

pub fn colored(_st: &str, _color: &str, _background: bool) {
    // def colored(st, color, background=False): return f"\u001b[{10*background+60*(color.upper() == color)+30+['black', 'red', 'green', 'yellow', 'blue', 'magenta', 'cyan', 'white'].index(color.lower())}m{st}\u001b[0m" if color is not None else st  # replace the termcolor library with one line
    todo!("Not yet implemented")
}

pub fn ansilen(_s: &str) {
    // def ansilen(s): return len(re.sub('\x1b\\[(K|.*?m)', '', s))
    todo!("Not yet implemented")
}

pub fn partition<T, F>(lst: &[T], fxn: F) -> (Vec<&T>, Vec<&T>)
where
    F: Fn(&T) -> bool,
{
    let x1: Vec<&T> = lst.iter().filter(|node| fxn(node)).collect();
    let x2: Vec<&T> = lst.iter().filter(|node| !fxn(node)).collect();
    (x1, x2)
}

pub fn gcd(mut a: isize, mut b: isize) -> isize {
    while b != 0 {
        let remainder = a % b;
        a = b;
        b = remainder;
    }
    a
}

pub fn make_pair(_x: &str, _cnt: &str) {
    // def make_pair(x:Union[int, Tuple[int, ...]], cnt=2) -> Tuple[int, ...]: return (x,)*cnt if isinstance(x, int) else x
    todo!("Not yet implemented")
}

pub fn flatten(_l: &str) {
    // def flatten(l:Iterator): return [item for sublist in l for item in sublist]
    todo!("Not yet implemented")
}

pub fn mnum(_i: &str) {
    // def mnum(i) -> str: return str(i) if i >= 0 else f"m{-i}"
    todo!("Not yet implemented")
}

pub fn fromimport() {
    // def fromimport(mod, frm): return getattr(__import__(mod, fromlist=[frm]), frm)
    todo!("Not yet implemented")
}

pub fn getenv() {
    // @functools.lru_cache(maxsize=None)
    // def getenv(key, default=0): return type(default)(os.getenv(key, default))
    todo!("Not yet implemented")
}

pub struct Context {
    // class Context:
    //   def __init__(self, **kwargs): self.pvars = kwargs
    //   def __enter__(self): ContextVar.ctx_stack.append({ **self.pvars, **{ key: ContextVar.ctx_stack[-1][key] for key in ContextVar.ctx_stack[-1].keys() if key not in self.pvars } })
    //   def __exit__(self, *args): ContextVar.ctx_stack.pop()
}

pub struct ContextVar {
    // class ContextVar:
    //   ctx_stack: ClassVar[List[dict[str, Any]]] = [{}]
    //   def __init__(self, key, default_value):
    //     self.key, self.initial_value = key, getenv(key, default_value)
    //     if key not in ContextVar.ctx_stack[-1]: ContextVar.ctx_stack[-1][key] = self.initial_value
    //   def __call__(self, x): ContextVar.ctx_stack[-1][self.key] = x
    //   def __bool__(self): return self.value != 0
    //   def __ge__(self, x): return self.value >= x
    //   def __gt__(self, x): return self.value > x
    //   def __lt__(self, x): return self.value < x
    //   @property
    //   def value(self): return ContextVar.ctx_stack[-1][self.key] if self.key in ContextVar.ctx_stack[-1] else self.initial_value
    // DEBUG, IMAGE = ContextVar("DEBUG", 0), ContextVar("IMAGE", 0)
    // GRAPH, PRUNEGRAPH, GRAPHPATH = getenv("GRAPH", 0), getenv("PRUNEGRAPH", 0), getenv("GRAPHPATH", "/tmp/net")
}

pub struct Timing {
    // class Timing(object):
    //   def __init__(self, prefix="", on_exit=None, enabled=True): self.prefix, self.on_exit, self.enabled = prefix, on_exit, enabled
    //   def __enter__(self): self.st = time.perf_counter_ns()
    //   def __exit__(self, exc_type, exc_val, exc_tb):
    //     self.et = time.perf_counter_ns() - self.st
    //     if self.enabled: print(f"{self.prefix}{self.et*1e-6:.2f} ms"+(self.on_exit(self.et) if self.on_exit else ""))
}

// # **** tinygrad now supports dtypes! *****

pub trait DType {
    // class DType(NamedTuple):
    //   priority: int  # this determines when things get upcasted
    //   itemsize: int
    //   name: str
    //   np: Optional[type]  # TODO: someday this will be removed with the "remove numpy" project
    //   sz: int = 1
    //   def __repr__(self): return f"dtypes.{self.name}"
    //   @property
    //   def key(self): return (self.name)
}

pub struct ImageDType {
    // # dependent typing?
    // class ImageDType(DType):
    //   def __new__(cls, priority, itemsize, name, np, shape):
    //     return super().__new__(cls, priority, itemsize, name, np)
    //   def __init__(self, priority, itemsize, name, np, shape):
    //     self.shape: Tuple[int, ...] = shape  # arbitrary arg for the dtype, used in image for the shape
    //     super().__init__()
    //   def __repr__(self): return f"dtypes.{self.name}({self.shape})"
}

impl DType for ImageDType {}

pub struct DTypes {
    // class dtypes:
    //   @staticmethod # static methds on top, or bool in the type info will refer to dtypes.bool
    //   def is_int(x: DType)-> bool: return x in (dtypes.int8, dtypes.uint8, dtypes.int32, dtypes.int64)
    //   @staticmethod
    //   def is_float(x: DType) -> bool: return x in (dtypes.float16, dtypes.float32, dtypes._half4, dtypes._float4)
    //   @staticmethod
    //   def is_unsigned(x: DType) -> bool: return x in (dtypes.uint8, dtypes.uint32, dtypes.uint64)
    //   @staticmethod
    //   def from_np(x) -> DType: return DTYPES_DICT[np.dtype(x).name]
    //   @staticmethod
    //   def fields() -> Dict[str, DType]: return DTYPES_DICT
    //   bool: Final[DType] = DType(0, 1, "bool", bool)
    //   float16: Final[DType] = DType(0, 2, "half", np.float16)
    //   half = float16
    //   float32: Final[DType] = DType(4, 4, "float", np.float32)
    //   float = float32
    //   int8: Final[DType] = DType(0, 1, "char", np.int8)
    //   int32: Final[DType] = DType(1, 4, "int", np.int32)
    //   int64: Final[DType] = DType(2, 8, "long", np.int64)
    //   uint8: Final[DType] = DType(0, 1, "unsigned char", np.uint8)
    //   uint32: Final[DType] = DType(1, 4, "unsigned int", np.uint32)
    //   uint64: Final[DType] = DType(2, 8, "unsigned long", np.uint64)
    //   # NOTE: these are internal dtypes, should probably check for that
    //   _half4: Final[DType] = DType(0, 2*4, "half4", None, 4)
    //   _float2: Final[DType] = DType(4, 4*2, "float2", None, 2)
    //   _float4: Final[DType] = DType(4, 4*4, "float4", None, 4)
    // # HACK: staticmethods are not callable in 3.8 so we have to compare the class
    // DTYPES_DICT = {k: v for k, v in dtypes.__dict__.items() if not k.startswith('__') and not callable(v) and not v.__class__ == staticmethod}
}

pub struct GlobalCounters {
    // class GlobalCounters:
    //   global_ops: ClassVar[int] = 0
    //   global_mem: ClassVar[int] = 0
    //   time_sum_s: ClassVar[float] = 0.0
    //   kernel_count: ClassVar[int] = 0
    //   mem_used: ClassVar[int] = 0   # NOTE: this is not reset
    //   cache: ClassVar[Optional[List[Tuple[Callable, Any]]]] = None
    //   @staticmethod
    //   def reset(): GlobalCounters.global_ops, GlobalCounters.global_mem, GlobalCounters.time_sum_s, GlobalCounters.kernel_count, GlobalCounters.cache = 0,0,0.0,0,None
}

pub struct LightWeakSet {
    // # Stripped down version of a WeakSet
    // class LightWeakSet:
    //   __slots__ = 'data', '_remove', '__weakref__'
    //   def __init__(self):
    //     self.data = set()
    //     def _remove(item, selfref=ref(self)):
    //       self = selfref()
    //       if self: self.data.discard(item)
    //     self._remove = _remove

    //   def __len__(self): return len(self.data)
    //   def add(self, item): self.data.add(ref(item, self._remove))
    //   def discard(self, item): self.data.discard(ref(item))
}

pub struct LightWeakValueDictionary {
    // # Stripped down version of a WeakValueDictionary
    // class LightWeakValueDictionary:
    //   __slots__ = 'data', '_remove', '__weakref__'
    //   def __init__(self):
    //     def remove(wr, selfref=ref(self), _atomic_removal=_remove_dead_weakref):
    //       self = selfref()
    //       if self: _atomic_removal(self.data, wr.key)
    //     self._remove = remove
    //     self.data = {}
    //   def __getitem__(self, key):
    //     o = self.data[key]()
    //     if o is None: raise KeyError(key)
    //     else: return o

    //   def __len__(self): return len(self.data)
    //   def __delitem__(self, key): del self.data[key]
    //   def __setitem__(self, key, value): self.data[key] = KeyedRef(value, self._remove, key)
    //   def __contains__(self, key): return key in self.data
}

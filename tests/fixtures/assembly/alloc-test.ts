


export function main(args_ptr: i32, args_len: i32): i64 {
  const RESULT_HEAP = heap.alloc(256);
  // Test more complex allocations that might trigger AS runtime issues
  const objects = new Array<Foo>(5);  // Array of objects
  let total = 0;

  // Create object graph similar to what anan-as might do
  for (let i = 0; i < 5; i++) {
    objects[i] = new Foo();
    objects[i].x = i * 10;
    const child = new Bar();
    child.value = i * 100;
    objects[i].child = child;

    // Create some circular references and complex relationships
    if (i > 0) {
      const prevChild = objects[i-1].child;
      if (prevChild) {
        prevChild.parent = objects[i-1];
      }
      objects[i-1].next = objects[i];
    }

    total += objects[i].x + child.value;
  }

  // Force some allocations that might trigger GC
  const tempArrays = new Array<Array<i32>>(3);
  for (let i = 0; i < 3; i++) {
    tempArrays[i] = new Array<i32>(i + 1);
    for (let j = 0; j <= i; j++) {
      tempArrays[i][j] = j * i;
      total += tempArrays[i][j];
    }
  }

  // Return result
  store<i32>(RESULT_HEAP, total);

  return (RESULT_HEAP as i64) | ((4 as i64) << 32);
}

class Foo {
  x: i32 = 0;
  child: Bar | null = null;
  next: Foo | null = null;
}

class Bar {
  value: i32 = 0;
  parent: Foo | null = null;
}
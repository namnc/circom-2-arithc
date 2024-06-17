// This circuit should fail because of out of bounds error

pragma circom 2.1.0;

template indexOutOfBounds() {
   signal arr[10];

   for (var i = 0; i < 100; i++) {
      arr[i] <== 1;
   }
}

component main = indexOutOfBounds();

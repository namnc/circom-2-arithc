pragma circom 2.0.0;

template InnerProd () {  

   // Declaration of signals 
   signal input input_A[3];  
   signal input input_B[3];  
   signal output ip;

   signal sum[3];

   sum[0] <== input_A[0]*input_B[0];

   // for (var i = 1; i < 3; i++) {
   //    sum[i] <== sum[i-1] + input_A[i] * input_B[i];
   // }

   // var sum = 0;

   // for (var i = 0; i < 3; i++) {
   //    sum = sum + input_A[i]*input_B[i];
   // }

   ip <== sum[2];
}

component main = InnerProd();

pragma circom 2.0.0;

function funid ( pa, pb ) {

 return pa;
}

template InnerProd (i) {  

   // Declaration of signals 
   signal input input_A[3];  
   signal input input_B[3];  
   signal output ip;

   signal sum[3];

   sum[0] <== input_A[0]*input_B[0];

   var z = funid (0,1);

   // var i = 0;
   // component ip2 = InnerProd(3);
   // ip2.input_A[0] <== input_A[0];
   // ip2.ip ==> ip;
   // if (i == 0) {
   //    sum[0] <== input_A[0]*input_B[0];
   // } else {
   //    sum[0] <== input_A[1]*input_B[1];
   // }

   // for (var i = 1; i < 3; i++) {
   //    sum[i] <== sum[i-1] + input_A[i] * input_B[i];
   // }

   // var sum = 0;

   // for (var i = 0; i < 3; i++) {
   //    sum = sum + input_A[i]*input_B[i];
   // }

   ip <== sum[2];
}

component main = InnerProd(1);

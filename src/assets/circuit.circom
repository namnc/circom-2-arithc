pragma circom 2.0.0;

template InnerProd () {  
   signal input input_A;  
   signal input input_B;  
   signal output ip;

   var variable_A;

   variable_A = 100;

   ip <== input_A + input_B + variable_A;
}

component main = InnerProd();

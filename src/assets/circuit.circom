pragma circom 2.0.0;

template Wire(){
   signal input input_signal[3];
   signal input input_signal_b[3];
   signal input input_signal_c[3];
   signal output output_signal;
   output_signal <== input_signal[1];
}

template Product () {  
   signal input input_A;  
   signal input input_B;  
   signal output ip;

   var variable_A;
   var variable_B;

   variable_A = 100;

   variable_B = is_positive(variable_A);

   component wire_component[2];

   wire_component[0] = Wire();

   wire_component[0].input_signal[1] <== 5;

   while (variable_A > 10) {
      variable_A = variable_A - 1;
   }

   ip <==  input_B + variable_B + variable_A + wire_component[0].input_signal[1];
}

function is_positive(n){
   if(n > 0){ return 50;}
   else{ return 0;}
}

component main = Product();

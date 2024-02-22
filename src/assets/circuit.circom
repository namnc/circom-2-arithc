pragma circom 2.0.0;

template Wire(){
   signal input input_signal;
   signal output output_signal;
   output_signal <== input_signal;
}

template MainComponent () {  
   signal input input_A;  
   signal input input_B;  
   signal output ip;

   var variable_A;
   var variable_B;

   variable_A = 100;
   variable_B = is_positive(variable_A);

   component wire_component;
   wire_component = Wire();

   wire_component.input_signal <== input_A + input_B;

   while (variable_A > 10) {
      variable_A = variable_A - 1;
   }

   ip <== input_A + input_B + variable_A + variable_B + wire_component.output_signal;
}

function is_positive(n){
   if(n > 0){ return 50;}
   else{ return 0;}
}

component main = MainComponent();

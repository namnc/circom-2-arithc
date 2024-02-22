pragma circom 2.0.0;

template MainComponent () {  
   signal input input_a;  
   signal output out;
   var variable_A;
   var variable_B;
   var variable_C;

   variable_A = 10;
   variable_B = 10;
   variable_C = 10;


   if (variable_A == 10) {
      var new_var = 10;
      variable_A = new_var;
   } else {
      var new_var_else = 20;
      variable_A = new_var_else;
   }

   out <== input_a + variable_A;
}

component main = MainComponent();

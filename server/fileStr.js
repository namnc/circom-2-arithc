
const file = `
pragma circom 2.0.0;

template libt (x) {  
    signal input input_A;  
    signal input input_B;  
    signal output ip;
 
    var variable_A;
 
    variable_A = x;
 
    ip <== input_A + input_B + variable_A;
 }
 
 function libf(x) {
     return 1 + x;
 }
 

template InnerProd () {  
   signal input input_A;  
   signal input input_B;  
   signal output ip;

   var variable_A;

   variable_A = 100;

   // for (var i = 0; i < 3; i++) {
   //    variable_A = variable_A + 10;
   // }

   // if ( variable_A < 50) {
   //    variable_A = variable_A * 2;
   // } else {
   //    variable_A = variable_A / 2;
   // }

   component c = libt(20);
   c.input_A <== input_A;
   c.input_B <== input_B;
   c.ip <== ip;

   ip <== input_A + input_B + variable_A;

   // variable_A = libf(variable_A);

   // component c = libt();
}

component main = InnerProd();

` 

console.log(JSON.stringify({file}));
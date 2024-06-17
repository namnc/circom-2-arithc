pragma circom 2.0.0;


template ShiftLeft(n) {
    signal input in;
    signal output out;
	
	out <== in << n;
}

template ShiftRight(n) {
    signal input in;
    signal output out;
	
	out <== in >> n;
}
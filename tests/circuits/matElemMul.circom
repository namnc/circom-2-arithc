pragma circom 2.1.0;

// Matrix multiplication by element
template matElemMul (m,n) {
    signal input a[m][n];
    signal input b[m][n];
    signal output out[m][n];
    
    for (var i=0; i < m; i++) {
        for (var j=0; j < n; j++) {
            out[i][j] <== a[i][j] * b[i][j];
        }
    }
}

component main = matElemMul(2,2);

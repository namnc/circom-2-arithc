pragma circom 2.1.0;

// M parties each with N items

template naive_search(M,N) {
   signal input inlist[M][N];
   signal input in;
   signal output out;
   signal matches[M][N];
   signal sum[M][N];

   sum[0][0] <== 0 + 0;
   for (var i = 1; i < N; i++) {
         matches[0][i] <== in == inlist[0][i];
         sum[0][i] <== sum[0][i-1] + matches[0][i];
      }

   for (var i = 1; i < M; i++) {
      sum[i][0] <== sum[i-1][N-1];
      matches[i][0] <== in == inlist[i][0];
      for (var j = 1; j < N; j++) {
         matches[i][j] <== in == inlist[i][j];
         sum[i][j] <== sum[i][j-1] + matches[i][j];
      }
   }
   

   
   out <== sum[M-1][N-1] + 2;
}

component main = naive_search(3,5);

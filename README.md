<div align="center">
  <img height="170x" src="https://vyperprotocol.io/wp-content/uploads/2021/12/vyp.png" />

  <h1>Vyper</h1>

  <p>
    <strong>The first tranching protocol in DeFi</strong>
  </p>

  <div align="left">
  <h2>Running the tests</h2>  
    Since Vyper integrates with <a href="https://github.com/project-serum/serum-dex">Serum DEX</a> in order to facilitate trades of the tranche tokens, the Serum DEX repository has been added as a submodule in the <code>deps/</code> sub-directory. To fetch the sub-module, run the following from the root of the cloned project.<br><br>
    
    git submodule update --init --recursive
   
   Then, build the Serum DEX program locally so that it can be deployed on the test validator and be interacted with.<br>
    
    cd deps/serum-dex/dex/ && cargo build-bpf && cd ../../../
    
   Assuming that you have Anchor setup, the tests can be run as follows.
    
    anchor test
  </div>

</div>

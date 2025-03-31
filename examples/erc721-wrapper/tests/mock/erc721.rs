#![allow(dead_code)]
#![cfg(feature = "e2e")]
use alloy::{primitives::Address, sol};
use e2e::Wallet;

sol! {
    #[allow(missing_docs)]
    // Built with Remix IDE; solc 0.8.24+commit.e11b9ed9
    #[sol(rpc, bytecode="608060405234801562000010575f80fd5b506040518060400160405280600a81526020017f4552433732314d6f636b000000000000000000000000000000000000000000008152506040518060400160405280600381526020017f4d544b0000000000000000000000000000000000000000000000000000000000815250815f90816200008d91906200030c565b5080600190816200009f91906200030c565b505050620003f0565b5f81519050919050565b7f4e487b71000000000000000000000000000000000000000000000000000000005f52604160045260245ffd5b7f4e487b71000000000000000000000000000000000000000000000000000000005f52602260045260245ffd5b5f60028204905060018216806200012457607f821691505b6020821081036200013a5762000139620000df565b5b50919050565b5f819050815f5260205f209050919050565b5f6020601f8301049050919050565b5f82821b905092915050565b5f600883026200019e7fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff8262000161565b620001aa868362000161565b95508019841693508086168417925050509392505050565b5f819050919050565b5f819050919050565b5f620001f4620001ee620001e884620001c2565b620001cb565b620001c2565b9050919050565b5f819050919050565b6200020f83620001d4565b620002276200021e82620001fb565b8484546200016d565b825550505050565b5f90565b6200023d6200022f565b6200024a81848462000204565b505050565b5b818110156200027157620002655f8262000233565b60018101905062000250565b5050565b601f821115620002c0576200028a8162000140565b620002958462000152565b81016020851015620002a5578190505b620002bd620002b48562000152565b8301826200024f565b50505b505050565b5f82821c905092915050565b5f620002e25f1984600802620002c5565b1980831691505092915050565b5f620002fc8383620002d1565b9150826002028217905092915050565b6200031782620000a8565b67ffffffffffffffff811115620003335762000332620000b2565b5b6200033f82546200010c565b6200034c82828562000275565b5f60209050601f83116001811462000382575f84156200036d578287015190505b620003798582620002ef565b865550620003e8565b601f198416620003928662000140565b5f5b82811015620003bb5784890151825560018201915060208501945060208101905062000394565b86831015620003db5784890151620003d7601f891682620002d1565b8355505b6001600288020188555050505b505050505050565b611d7780620003fe5f395ff3fe608060405234801561000f575f80fd5b50600436106100e8575f3560e01c806370a082311161008a578063a22cb46511610064578063a22cb46514610258578063b88d4fde14610274578063c87b56dd14610290578063e985e9c5146102c0576100e8565b806370a08231146101ee57806395d89b411461021e578063a14481941461023c576100e8565b8063095ea7b3116100c6578063095ea7b31461016a57806323b872dd1461018657806342842e0e146101a25780636352211e146101be576100e8565b806301ffc9a7146100ec57806306fdde031461011c578063081812fc1461013a575b5f80fd5b61010660048036038101906101019190611608565b6102f0565b604051610113919061164d565b60405180910390f35b6101246103d1565b60405161013191906116f0565b60405180910390f35b610154600480360381019061014f9190611743565b610460565b60405161016191906117ad565b60405180910390f35b610184600480360381019061017f91906117f0565b61047b565b005b6101a0600480360381019061019b919061182e565b610489565b005b6101bc60048036038101906101b7919061182e565b610588565b005b6101d860048036038101906101d39190611743565b6105a7565b6040516101e591906117ad565b60405180910390f35b6102086004803603810190610203919061187e565b6105b8565b60405161021591906118b8565b60405180910390f35b61022661066e565b60405161023391906116f0565b60405180910390f35b610256600480360381019061025191906117f0565b6106fe565b005b610272600480360381019061026d91906118fb565b61070c565b005b61028e60048036038101906102899190611a65565b610722565b005b6102aa60048036038101906102a59190611743565b610747565b6040516102b791906116f0565b60405180910390f35b6102da60048036038101906102d59190611ae5565b6107ad565b6040516102e7919061164d565b60405180910390f35b5f7f80ac58cd000000000000000000000000000000000000000000000000000000007bffffffffffffffffffffffffffffffffffffffffffffffffffffffff1916827bffffffffffffffffffffffffffffffffffffffffffffffffffffffff191614806103ba57507f5b5e139f000000000000000000000000000000000000000000000000000000007bffffffffffffffffffffffffffffffffffffffffffffffffffffffff1916827bffffffffffffffffffffffffffffffffffffffffffffffffffffffff1916145b806103ca57506103c98261083b565b5b9050919050565b60605f80546103df90611b50565b80601f016020809104026020016040519081016040528092919081815260200182805461040b90611b50565b80156104565780601f1061042d57610100808354040283529160200191610456565b820191905f5260205f20905b81548152906001019060200180831161043957829003601f168201915b5050505050905090565b5f61046a826108a4565b506104748261092a565b9050919050565b6104858282610963565b5050565b5f73ffffffffffffffffffffffffffffffffffffffff168273ffffffffffffffffffffffffffffffffffffffff16036104f9575f6040517f64a0ae920000000000000000000000000000000000000000000000000000000081526004016104f091906117ad565b60405180910390fd5b5f61050c8383610507610979565b610980565b90508373ffffffffffffffffffffffffffffffffffffffff168173ffffffffffffffffffffffffffffffffffffffff1614610582578382826040517f64283d7b00000000000000000000000000000000000000000000000000000000815260040161057993929190611b80565b60405180910390fd5b50505050565b6105a283838360405180602001604052805f815250610722565b505050565b5f6105b182610b8b565b9050919050565b5f8073ffffffffffffffffffffffffffffffffffffffff168273ffffffffffffffffffffffffffffffffffffffff1603610629575f6040517f89c62b6400000000000000000000000000000000000000000000000000000000815260040161062091906117ad565b60405180910390fd5b60035f8373ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff1681526020019081526020015f20549050919050565b60606001805461067d90611b50565b80601f01602080910402602001604051908101604052809291908181526020018280546106a990611b50565b80156106f45780601f106106cb576101008083540402835291602001916106f4565b820191905f5260205f20905b8154815290600101906020018083116106d757829003601f168201915b5050505050905090565b6107088282610b9c565b5050565b61071e610717610979565b8383610bb9565b5050565b61072d848484610489565b610741610738610979565b85858585610d22565b50505050565b6060610752826108a4565b505f61075c610ece565b90505f81511161077a5760405180602001604052805f8152506107a5565b8061078484610ee4565b604051602001610795929190611bef565b6040516020818303038152906040525b915050919050565b5f60055f8473ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff1681526020019081526020015f205f8373ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff1681526020019081526020015f205f9054906101000a900460ff16905092915050565b5f7f01ffc9a7000000000000000000000000000000000000000000000000000000007bffffffffffffffffffffffffffffffffffffffffffffffffffffffff1916827bffffffffffffffffffffffffffffffffffffffffffffffffffffffff1916149050919050565b5f806108af83610fae565b90505f73ffffffffffffffffffffffffffffffffffffffff168173ffffffffffffffffffffffffffffffffffffffff160361092157826040517f7e27328900000000000000000000000000000000000000000000000000000000815260040161091891906118b8565b60405180910390fd5b80915050919050565b5f60045f8381526020019081526020015f205f9054906101000a900473ffffffffffffffffffffffffffffffffffffffff169050919050565b6109758282610970610979565b610fe7565b5050565b5f33905090565b5f8061098b84610fae565b90505f73ffffffffffffffffffffffffffffffffffffffff168373ffffffffffffffffffffffffffffffffffffffff16146109cc576109cb818486610ff9565b5b5f73ffffffffffffffffffffffffffffffffffffffff168173ffffffffffffffffffffffffffffffffffffffff1614610a5757610a0b5f855f806110bc565b600160035f8373ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff1681526020019081526020015f205f82825403925050819055505b5f73ffffffffffffffffffffffffffffffffffffffff168573ffffffffffffffffffffffffffffffffffffffff1614610ad657600160035f8773ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff1681526020019081526020015f205f82825401925050819055505b8460025f8681526020019081526020015f205f6101000a81548173ffffffffffffffffffffffffffffffffffffffff021916908373ffffffffffffffffffffffffffffffffffffffff160217905550838573ffffffffffffffffffffffffffffffffffffffff168273ffffffffffffffffffffffffffffffffffffffff167fddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3ef60405160405180910390a4809150509392505050565b5f610b95826108a4565b9050919050565b610bb5828260405180602001604052805f81525061127b565b5050565b5f73ffffffffffffffffffffffffffffffffffffffff168273ffffffffffffffffffffffffffffffffffffffff1603610c2957816040517f5b08ba18000000000000000000000000000000000000000000000000000000008152600401610c2091906117ad565b60405180910390fd5b8060055f8573ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff1681526020019081526020015f205f8473ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff1681526020019081526020015f205f6101000a81548160ff0219169083151502179055508173ffffffffffffffffffffffffffffffffffffffff168373ffffffffffffffffffffffffffffffffffffffff167f17307eab39ab6107e8899845ad3d59bd9653f200f220920489ca2b5937696c3183604051610d15919061164d565b60405180910390a3505050565b5f8373ffffffffffffffffffffffffffffffffffffffff163b1115610ec7578273ffffffffffffffffffffffffffffffffffffffff1663150b7a02868685856040518563ffffffff1660e01b8152600401610d809493929190611c64565b6020604051808303815f875af1925050508015610dbb57506040513d601f19601f82011682018060405250810190610db89190611cc2565b60015b610e3c573d805f8114610de9576040519150601f19603f3d011682016040523d82523d5f602084013e610dee565b606091505b505f815103610e3457836040517f64a0ae92000000000000000000000000000000000000000000000000000000008152600401610e2b91906117ad565b60405180910390fd5b805181602001fd5b63150b7a0260e01b7bffffffffffffffffffffffffffffffffffffffffffffffffffffffff1916817bffffffffffffffffffffffffffffffffffffffffffffffffffffffff191614610ec557836040517f64a0ae92000000000000000000000000000000000000000000000000000000008152600401610ebc91906117ad565b60405180910390fd5b505b5050505050565b606060405180602001604052805f815250905090565b60605f6001610ef28461129e565b0190505f8167ffffffffffffffff811115610f1057610f0f611941565b5b6040519080825280601f01601f191660200182016040528015610f425781602001600182028036833780820191505090505b5090505f82602001820190505b600115610fa3578080600190039150507f3031323334353637383961626364656600000000000000000000000000000000600a86061a8153600a8581610f9857610f97611ced565b5b0494505f8503610f4f575b819350505050919050565b5f60025f8381526020019081526020015f205f9054906101000a900473ffffffffffffffffffffffffffffffffffffffff169050919050565b610ff483838360016110bc565b505050565b6110048383836113ef565b6110b7575f73ffffffffffffffffffffffffffffffffffffffff168373ffffffffffffffffffffffffffffffffffffffff160361107857806040517f7e27328900000000000000000000000000000000000000000000000000000000815260040161106f91906118b8565b60405180910390fd5b81816040517f177e802f0000000000000000000000000000000000000000000000000000000081526004016110ae929190611d1a565b60405180910390fd5b505050565b80806110f457505f73ffffffffffffffffffffffffffffffffffffffff168273ffffffffffffffffffffffffffffffffffffffff1614155b15611226575f611103846108a4565b90505f73ffffffffffffffffffffffffffffffffffffffff168373ffffffffffffffffffffffffffffffffffffffff161415801561116d57508273ffffffffffffffffffffffffffffffffffffffff168173ffffffffffffffffffffffffffffffffffffffff1614155b8015611180575061117e81846107ad565b155b156111c257826040517fa9fbf51f0000000000000000000000000000000000000000000000000000000081526004016111b991906117ad565b60405180910390fd5b811561122457838573ffffffffffffffffffffffffffffffffffffffff168273ffffffffffffffffffffffffffffffffffffffff167f8c5be1e5ebec7d5bd14f71427d1e84f3dd0314c0f7b2291e5b200ac8c7c3b92560405160405180910390a45b505b8360045f8581526020019081526020015f205f6101000a81548173ffffffffffffffffffffffffffffffffffffffff021916908373ffffffffffffffffffffffffffffffffffffffff16021790555050505050565b61128583836114af565b611299611290610979565b5f858585610d22565b505050565b5f805f90507a184f03e93ff9f4daa797ed6e38ed64bf6a1f01000000000000000083106112fa577a184f03e93ff9f4daa797ed6e38ed64bf6a1f01000000000000000083816112f0576112ef611ced565b5b0492506040810190505b6d04ee2d6d415b85acef81000000008310611337576d04ee2d6d415b85acef8100000000838161132d5761132c611ced565b5b0492506020810190505b662386f26fc10000831061136657662386f26fc10000838161135c5761135b611ced565b5b0492506010810190505b6305f5e100831061138f576305f5e100838161138557611384611ced565b5b0492506008810190505b61271083106113b45761271083816113aa576113a9611ced565b5b0492506004810190505b606483106113d757606483816113cd576113cc611ced565b5b0492506002810190505b600a83106113e6576001810190505b80915050919050565b5f8073ffffffffffffffffffffffffffffffffffffffff168373ffffffffffffffffffffffffffffffffffffffff16141580156114a657508273ffffffffffffffffffffffffffffffffffffffff168473ffffffffffffffffffffffffffffffffffffffff161480611467575061146684846107ad565b5b806114a557508273ffffffffffffffffffffffffffffffffffffffff1661148d8361092a565b73ffffffffffffffffffffffffffffffffffffffff16145b5b90509392505050565b5f73ffffffffffffffffffffffffffffffffffffffff168273ffffffffffffffffffffffffffffffffffffffff160361151f575f6040517f64a0ae9200000000000000000000000000000000000000000000000000000000815260040161151691906117ad565b60405180910390fd5b5f61152b83835f610980565b90505f73ffffffffffffffffffffffffffffffffffffffff168173ffffffffffffffffffffffffffffffffffffffff161461159d575f6040517f73c6ac6e00000000000000000000000000000000000000000000000000000000815260040161159491906117ad565b60405180910390fd5b505050565b5f604051905090565b5f80fd5b5f80fd5b5f7fffffffff0000000000000000000000000000000000000000000000000000000082169050919050565b6115e7816115b3565b81146115f1575f80fd5b50565b5f81359050611602816115de565b92915050565b5f6020828403121561161d5761161c6115ab565b5b5f61162a848285016115f4565b91505092915050565b5f8115159050919050565b61164781611633565b82525050565b5f6020820190506116605f83018461163e565b92915050565b5f81519050919050565b5f82825260208201905092915050565b5f5b8381101561169d578082015181840152602081019050611682565b5f8484015250505050565b5f601f19601f8301169050919050565b5f6116c282611666565b6116cc8185611670565b93506116dc818560208601611680565b6116e5816116a8565b840191505092915050565b5f6020820190508181035f83015261170881846116b8565b905092915050565b5f819050919050565b61172281611710565b811461172c575f80fd5b50565b5f8135905061173d81611719565b92915050565b5f60208284031215611758576117576115ab565b5b5f6117658482850161172f565b91505092915050565b5f73ffffffffffffffffffffffffffffffffffffffff82169050919050565b5f6117978261176e565b9050919050565b6117a78161178d565b82525050565b5f6020820190506117c05f83018461179e565b92915050565b6117cf8161178d565b81146117d9575f80fd5b50565b5f813590506117ea816117c6565b92915050565b5f8060408385031215611806576118056115ab565b5b5f611813858286016117dc565b92505060206118248582860161172f565b9150509250929050565b5f805f60608486031215611845576118446115ab565b5b5f611852868287016117dc565b9350506020611863868287016117dc565b92505060406118748682870161172f565b9150509250925092565b5f60208284031215611893576118926115ab565b5b5f6118a0848285016117dc565b91505092915050565b6118b281611710565b82525050565b5f6020820190506118cb5f8301846118a9565b92915050565b6118da81611633565b81146118e4575f80fd5b50565b5f813590506118f5816118d1565b92915050565b5f8060408385031215611911576119106115ab565b5b5f61191e858286016117dc565b925050602061192f858286016118e7565b9150509250929050565b5f80fd5b5f80fd5b7f4e487b71000000000000000000000000000000000000000000000000000000005f52604160045260245ffd5b611977826116a8565b810181811067ffffffffffffffff8211171561199657611995611941565b5b80604052505050565b5f6119a86115a2565b90506119b4828261196e565b919050565b5f67ffffffffffffffff8211156119d3576119d2611941565b5b6119dc826116a8565b9050602081019050919050565b828183375f83830152505050565b5f611a09611a04846119b9565b61199f565b905082815260208101848484011115611a2557611a2461193d565b5b611a308482856119e9565b509392505050565b5f82601f830112611a4c57611a4b611939565b5b8135611a5c8482602086016119f7565b91505092915050565b5f805f8060808587031215611a7d57611a7c6115ab565b5b5f611a8a878288016117dc565b9450506020611a9b878288016117dc565b9350506040611aac8782880161172f565b925050606085013567ffffffffffffffff811115611acd57611acc6115af565b5b611ad987828801611a38565b91505092959194509250565b5f8060408385031215611afb57611afa6115ab565b5b5f611b08858286016117dc565b9250506020611b19858286016117dc565b9150509250929050565b7f4e487b71000000000000000000000000000000000000000000000000000000005f52602260045260245ffd5b5f6002820490506001821680611b6757607f821691505b602082108103611b7a57611b79611b23565b5b50919050565b5f606082019050611b935f83018661179e565b611ba060208301856118a9565b611bad604083018461179e565b949350505050565b5f81905092915050565b5f611bc982611666565b611bd38185611bb5565b9350611be3818560208601611680565b80840191505092915050565b5f611bfa8285611bbf565b9150611c068284611bbf565b91508190509392505050565b5f81519050919050565b5f82825260208201905092915050565b5f611c3682611c12565b611c408185611c1c565b9350611c50818560208601611680565b611c59816116a8565b840191505092915050565b5f608082019050611c775f83018761179e565b611c84602083018661179e565b611c9160408301856118a9565b8181036060830152611ca38184611c2c565b905095945050505050565b5f81519050611cbc816115de565b92915050565b5f60208284031215611cd757611cd66115ab565b5b5f611ce484828501611cae565b91505092915050565b7f4e487b71000000000000000000000000000000000000000000000000000000005f52601260045260245ffd5b5f604082019050611d2d5f83018561179e565b611d3a60208301846118a9565b939250505056fea264697066735822122076bd71be3fdee3ce4ce4a172354a2fae2f744c0c2690c688693e6e1cf07dea0a64736f6c63430008180033")]
    // SPDX-License-Identifier: MIT
    // Compatible with OpenZeppelin Contracts ^5.0.0
    contract ERC721Mock is ERC721 {
        constructor()
            ERC721("ERC721Mock", "MTK") {}

        function safeMint(address to, uint256 tokenId) public {
            _safeMint(to, tokenId);
        }

        function ownerOf(uint256 tokenId) public view override returns (address) {
            return super.ownerOf(tokenId);
        }

        function approve(address to, uint256 tokenId) public override  {
            super.approve(to, tokenId);
        }

        function safeTransferFrom(address from, address to, uint256 tokenId, bytes memory data) public override {
            super.safeTransferFrom(from, to, tokenId, data);
        }

        function balanceOf(address owner) public view override returns (uint256) {
            return super.balanceOf(owner);
        }
    }
}

pub async fn deploy(wallet: &Wallet) -> eyre::Result<Address> {
    // Deploy the contract.
    let contract = ERC721Mock::deploy(wallet).await?;
    Ok(*contract.address())
}

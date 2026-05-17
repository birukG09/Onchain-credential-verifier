import { ethers } from "hardhat";

async function main() {
  const [deployer] = await ethers.getSigners();

  console.log("Deploying CredentialNFT with account:", deployer.address);
  console.log(
    "Account balance:",
    ethers.formatEther(await ethers.provider.getBalance(deployer.address)),
    "MATIC"
  );

  const CredentialNFT = await ethers.getContractFactory("CredentialNFT");
  const contract = await CredentialNFT.deploy();
  await contract.waitForDeployment();

  const address = await contract.getAddress();
  console.log("CredentialNFT deployed to:", address);
  console.log("Set CONTRACT_ADDRESS=" + address + " in your .env file");

  const tx = await contract.addIssuer(deployer.address);
  await tx.wait();
  console.log("Deployer added as authorized issuer:", deployer.address);

  console.log("\n=== Deployment Summary ===");
  console.log("Contract Address:", address);
  console.log("Network:", (await ethers.provider.getNetwork()).name);
  console.log("Deployer:", deployer.address);
}

main().catch((err) => {
  console.error(err);
  process.exitCode = 1;
});

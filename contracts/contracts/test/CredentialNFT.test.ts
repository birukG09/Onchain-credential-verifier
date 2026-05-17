import { expect } from "chai";
import { ethers } from "hardhat";
import { CredentialNFT } from "../typechain-types";
import { SignerWithAddress } from "@nomicfoundation/hardhat-ethers/signers";

describe("CredentialNFT", function () {
  let contract: CredentialNFT;
  let owner: SignerWithAddress;
  let issuer: SignerWithAddress;
  let holder: SignerWithAddress;
  let verifier: SignerWithAddress;

  const SAMPLE_URI = "ipfs://QmSampleHash123/credential.json";
  const CRED_TYPE = "education";

  beforeEach(async () => {
    [owner, issuer, holder, verifier] = await ethers.getSigners();
    const Factory = await ethers.getContractFactory("CredentialNFT");
    contract = (await Factory.deploy()) as CredentialNFT;
    await contract.waitForDeployment();
  });

  describe("Deployment", () => {
    it("sets the correct owner", async () => {
      expect(await contract.owner()).to.equal(owner.address);
    });

    it("has the correct name and symbol", async () => {
      expect(await contract.name()).to.equal("CredentialNFT");
      expect(await contract.symbol()).to.equal("CRED");
    });

    it("starts with zero supply", async () => {
      expect(await contract.totalSupply()).to.equal(0n);
    });
  });

  describe("Issuer Management", () => {
    it("allows owner to add an issuer", async () => {
      await expect(contract.addIssuer(issuer.address))
        .to.emit(contract, "IssuerAdded")
        .withArgs(issuer.address);
      expect(await contract.authorizedIssuers(issuer.address)).to.equal(true);
    });

    it("allows owner to remove an issuer", async () => {
      await contract.addIssuer(issuer.address);
      await expect(contract.removeIssuer(issuer.address))
        .to.emit(contract, "IssuerRemoved")
        .withArgs(issuer.address);
      expect(await contract.authorizedIssuers(issuer.address)).to.equal(false);
    });

    it("reverts when non-owner tries to add issuer", async () => {
      await expect(
        contract.connect(holder).addIssuer(issuer.address)
      ).to.be.revertedWithCustomError(contract, "OwnableUnauthorizedAccount");
    });
  });

  describe("Credential Minting", () => {
    beforeEach(async () => {
      await contract.addIssuer(issuer.address);
    });

    it("mints a credential NFT", async () => {
      await expect(
        contract.connect(issuer).mintCredential(holder.address, CRED_TYPE, SAMPLE_URI)
      )
        .to.emit(contract, "CredentialMinted")
        .withArgs(0n, holder.address, issuer.address, CRED_TYPE, SAMPLE_URI);

      expect(await contract.ownerOf(0n)).to.equal(holder.address);
      expect(await contract.tokenURI(0n)).to.equal(SAMPLE_URI);
      expect(await contract.totalSupply()).to.equal(1n);
    });

    it("stores credential info correctly", async () => {
      await contract.connect(issuer).mintCredential(holder.address, CRED_TYPE, SAMPLE_URI);
      const info = await contract.getCredential(0n);

      expect(info.holder).to.equal(holder.address);
      expect(info.issuer).to.equal(issuer.address);
      expect(info.credentialType).to.equal(CRED_TYPE);
      expect(info.revoked).to.equal(false);
    });

    it("reverts when unauthorized account tries to mint", async () => {
      await expect(
        contract.connect(verifier).mintCredential(holder.address, CRED_TYPE, SAMPLE_URI)
      ).to.be.revertedWith("CredentialNFT: not an authorized issuer");
    });

    it("increments token IDs sequentially", async () => {
      await contract.connect(issuer).mintCredential(holder.address, CRED_TYPE, SAMPLE_URI);
      await contract.connect(issuer).mintCredential(holder.address, "certification", SAMPLE_URI);
      expect(await contract.totalSupply()).to.equal(2n);
      expect(await contract.ownerOf(0n)).to.equal(holder.address);
      expect(await contract.ownerOf(1n)).to.equal(holder.address);
    });

    it("owner can also mint without being added as issuer", async () => {
      await expect(
        contract.connect(owner).mintCredential(holder.address, CRED_TYPE, SAMPLE_URI)
      ).to.not.be.reverted;
    });
  });

  describe("Credential Validity", () => {
    beforeEach(async () => {
      await contract.addIssuer(issuer.address);
      await contract.connect(issuer).mintCredential(holder.address, CRED_TYPE, SAMPLE_URI);
    });

    it("returns true for a valid credential", async () => {
      expect(await contract.isCredentialValid(0n)).to.equal(true);
    });

    it("returns false for a non-existent token", async () => {
      expect(await contract.isCredentialValid(999n)).to.equal(false);
    });
  });

  describe("Credential Revocation", () => {
    beforeEach(async () => {
      await contract.addIssuer(issuer.address);
      await contract.connect(issuer).mintCredential(holder.address, CRED_TYPE, SAMPLE_URI);
    });

    it("issuer can revoke a credential", async () => {
      await expect(contract.connect(issuer).revokeCredential(0n))
        .to.emit(contract, "CredentialRevoked")
        .withArgs(0n, issuer.address);

      expect(await contract.isCredentialValid(0n)).to.equal(false);
      expect(await contract.revokedTokens(0n)).to.equal(true);
    });

    it("owner can revoke any credential", async () => {
      await expect(contract.connect(owner).revokeCredential(0n)).to.not.be.reverted;
      expect(await contract.isCredentialValid(0n)).to.equal(false);
    });

    it("reverts when unauthorized account tries to revoke", async () => {
      await expect(
        contract.connect(verifier).revokeCredential(0n)
      ).to.be.revertedWith("CredentialNFT: not authorized to revoke");
    });

    it("reverts when revoking an already revoked credential", async () => {
      await contract.connect(issuer).revokeCredential(0n);
      await expect(
        contract.connect(issuer).revokeCredential(0n)
      ).to.be.revertedWith("CredentialNFT: already revoked");
    });
  });

  describe("Employer Verification", () => {
    beforeEach(async () => {
      await contract.addIssuer(issuer.address);
      await contract.connect(issuer).mintCredential(holder.address, CRED_TYPE, SAMPLE_URI);
    });

    it("emits CredentialVerified event on verify call", async () => {
      await expect(contract.connect(verifier).verifyCredential(0n))
        .to.emit(contract, "CredentialVerified")
        .withArgs(0n, verifier.address, true);
    });

    it("emits false for revoked credential", async () => {
      await contract.connect(issuer).revokeCredential(0n);
      await expect(contract.connect(verifier).verifyCredential(0n))
        .to.emit(contract, "CredentialVerified")
        .withArgs(0n, verifier.address, false);
    });
  });
});

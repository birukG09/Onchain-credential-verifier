// SPDX-License-Identifier: MIT
pragma solidity ^0.8.24;

import "@openzeppelin/contracts/token/ERC721/extensions/ERC721URIStorage.sol";
import "@openzeppelin/contracts/access/Ownable.sol";

/// @title CredentialNFT
/// @notice ERC721 contract for minting verifiable on-chain credentials
/// @dev Supports education, certifications, and employment history as NFTs
contract CredentialNFT is ERC721URIStorage, Ownable {
    uint256 private _tokenIdCounter;

    /// @notice Whitelisted issuers authorized to mint credentials
    mapping(address => bool) public authorizedIssuers;

    /// @notice Tracks revoked token IDs
    mapping(uint256 => bool) public revokedTokens;

    struct CredentialInfo {
        address holder;
        address issuer;
        string credentialType;
        uint256 issuedAt;
        bool revoked;
    }

    mapping(uint256 => CredentialInfo) public credentials;

    // ── Events ──────────────────────────────────────────────────────────────

    event CredentialMinted(
        uint256 indexed tokenId,
        address indexed holder,
        address indexed issuer,
        string credentialType,
        string tokenURI
    );

    event CredentialRevoked(uint256 indexed tokenId, address indexed revoker);
    event IssuerAdded(address indexed issuer);
    event IssuerRemoved(address indexed issuer);
    event CredentialVerified(uint256 indexed tokenId, address indexed verifier, bool valid);

    // ── Constructor ──────────────────────────────────────────────────────────

    constructor() ERC721("CredentialNFT", "CRED") Ownable(msg.sender) {}

    // ── Modifiers ────────────────────────────────────────────────────────────

    modifier onlyIssuer() {
        require(
            authorizedIssuers[msg.sender] || owner() == msg.sender,
            "CredentialNFT: not an authorized issuer"
        );
        _;
    }

    // ── Issuer Management ────────────────────────────────────────────────────

    /// @notice Add an authorized issuer
    function addIssuer(address issuer) external onlyOwner {
        require(issuer != address(0), "CredentialNFT: zero address");
        authorizedIssuers[issuer] = true;
        emit IssuerAdded(issuer);
    }

    /// @notice Remove an authorized issuer
    function removeIssuer(address issuer) external onlyOwner {
        authorizedIssuers[issuer] = false;
        emit IssuerRemoved(issuer);
    }

    // ── Credential Minting ───────────────────────────────────────────────────

    /// @notice Mint a new credential NFT
    /// @param holder Address receiving the credential
    /// @param credentialType Category (e.g. "education", "certification", "employment")
    /// @param tokenURI IPFS URI pointing to the credential metadata JSON
    function mintCredential(
        address holder,
        string calldata credentialType,
        string calldata tokenURI
    ) external onlyIssuer returns (uint256) {
        require(holder != address(0), "CredentialNFT: mint to zero address");
        require(bytes(credentialType).length > 0, "CredentialNFT: empty credential type");
        require(bytes(tokenURI).length > 0, "CredentialNFT: empty token URI");

        uint256 tokenId = _tokenIdCounter;
        _tokenIdCounter++;

        _safeMint(holder, tokenId);
        _setTokenURI(tokenId, tokenURI);

        credentials[tokenId] = CredentialInfo({
            holder: holder,
            issuer: msg.sender,
            credentialType: credentialType,
            issuedAt: block.timestamp,
            revoked: false
        });

        emit CredentialMinted(tokenId, holder, msg.sender, credentialType, tokenURI);
        return tokenId;
    }

    // ── Revocation ───────────────────────────────────────────────────────────

    /// @notice Revoke a credential — callable by original issuer or contract owner
    function revokeCredential(uint256 tokenId) external {
        CredentialInfo storage cred = credentials[tokenId];
        require(_ownerOf(tokenId) != address(0), "CredentialNFT: token does not exist");
        require(
            cred.issuer == msg.sender || owner() == msg.sender,
            "CredentialNFT: not authorized to revoke"
        );
        require(!cred.revoked, "CredentialNFT: already revoked");

        cred.revoked = true;
        revokedTokens[tokenId] = true;

        emit CredentialRevoked(tokenId, msg.sender);
    }

    // ── Verification ─────────────────────────────────────────────────────────

    /// @notice Check whether a credential is valid (exists and not revoked)
    function isCredentialValid(uint256 tokenId) external view returns (bool) {
        if (_ownerOf(tokenId) == address(0)) return false;
        return !credentials[tokenId].revoked;
    }

    /// @notice Emit a verification event (for employer/verifier logging)
    function verifyCredential(uint256 tokenId) external {
        bool valid = _ownerOf(tokenId) != address(0) && !credentials[tokenId].revoked;
        emit CredentialVerified(tokenId, msg.sender, valid);
    }

    /// @notice Return full credential info struct
    function getCredential(uint256 tokenId) external view returns (CredentialInfo memory) {
        require(_ownerOf(tokenId) != address(0), "CredentialNFT: token does not exist");
        return credentials[tokenId];
    }

    /// @notice Return total minted token count
    function totalSupply() external view returns (uint256) {
        return _tokenIdCounter;
    }
}

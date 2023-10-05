///
/// @file zkAppUtilsClass.h
/// @author Scott Miller
/// @version 1.0
/// @date November 17, 2016
/// @copyright Zymbit, Inc.
/// @brief C++ interface to Zymkey Application Utilities Library.
/// @details
/// This file contains the C API to the the Zymkey Application Utilities
/// library. This API facilitates writing user space applications which use
/// Zymkey to perform cryptographic operations, such as:
///      1. Signing of payloads using ECDSA
///      2. Verification of payloads that were signed using Zymkey
///      3. Exporting the public key that matches Zymkey's private key
///      4. "Locking" and "unlocking" data objects
///      5. Generating random data
/// Additionally, there are functions for changing the i2c address (i2c units
/// only), setting tap sensitivity and controlling the LED.
///

#ifndef ZKAPPUTILSCLASS_H
#define ZKAPPUTILSCLASS_H

#include <zk_app_utils.h>
#include <exception>
#include <string>
#include <vector>

namespace zkAppUtils
{

///
/// @brief Exception class derived from std::exception
///
class commonException: public std::exception
{
    public:
        ///
        /// @brief Constructor
        /// @param status
        ///        A string which described the verbose exception
        commonException(std::string status);
        ///
        /// @brief Destructor
        ///
        virtual ~commonException() throw() {}
        ///
        /// @brief Override of std::exception what() method
        ///
        const char* what() const throw();
    private:
        ///
        /// @var Exception message
        ///
        std::string msg;
};

///
/// @brief Exception class derived from std::exception
///
class timeoutException: public std::exception
{
    public:
        ///
        /// @brief Constructor
        /// @param status
        ///        A string which described the verbose exception
        timeoutException(std::string status);
        ///
        /// @brief Destructor
        ///
        virtual ~timeoutException() throw() {}
        ///
        /// @brief Override of std::exception what() method
        ///
        const char* what() const throw();
    private:
        ///
        /// @var Exception message
        ///
        std::string msg;
};

///
/// @brief Class used for retrieval of recovery mnemonic for Bip32 wallet
///
class recoveryStrategyBaseType
{
   protected:
      ZK_RECOVERY_STRATEGY recoveryStrategy;
      std::string variant;

   public:
      recoveryStrategyBaseType(const std::string& key_variant = "");
      ~recoveryStrategyBaseType();

      ZK_RECOVERY_STRATEGY getRecoveryStrategy() const { return recoveryStrategy; }

      void setVariant(const std::string& key_variant){ variant = key_variant; }
      std::string getVariant() const { return variant; }

      virtual std::string getPassPhrase() const { return ""; }
};

///
/// @brief Class used for BIP39 recovery strategy. User can set b64 encoded passphrase and
/// key variant (only "cardano" is supported for ed25519 currently) for bip32 seed generation.
/// It can also hold the mnemonic sentence from master seed generation.
///
class recoveryStrategyBIP39 : public recoveryStrategyBaseType
{
   private:
      ///
      /// @var Base 64 encoded string
      ///
      std::string passPhrase;

   public:
      //Defaults
      recoveryStrategyBIP39(const std::string& passphrase = "",
                            const std::string& key_variant = "");
      ~recoveryStrategyBIP39();

      void setPassPhrase(const std::string& passphrase){ passPhrase = passphrase; }
      std::string getPassPhrase() const { return passPhrase; }
};

class recoveryStrategySLIP39: public recoveryStrategyBaseType
{
   private:
      int groupCount;
      int groupThreshold;
      int iterationExponent;
      std::string passPhrase;

   public:
      //Defaults
      recoveryStrategySLIP39(int group_count,
                             int group_threshold,
                             int iteration_exponent,
                             const std::string& passphrase = "",
                             const std::string& key_variant = "");
      ~recoveryStrategySLIP39();

      void setPassPhrase(const std::string& passphrase){ passPhrase = passphrase; }
      std::string getPassPhrase() const { return passPhrase; }

      void setGroupCount(int group_count){ groupCount = group_count; }
      int getGroupCount() const { return groupCount; }

      void setGroupThreshold(int group_threshold){ groupThreshold = group_threshold; }
      int getGroupThreshold() const { return groupThreshold; }

      void setIterationExponent(int iteration_exponent){ iterationExponent = iteration_exponent; }
      int getIterationExponent() const { return iterationExponent; }
};

///
/// @brief Structure typedef used for retrieval of accelerometer data
///
struct accelData
{
    double x;
    double y;
    double z;
    int tapDirX;
    int tapDirY;
    int tapDirZ;
};

///
/// @brief Typedef for a simple unsigned bytes container
///
typedef std::vector<uint8_t> byteArray;

///
/// @brief Typedef for a simple integer container
///
typedef std::vector<int> intArray;

///
/// @brief The main class
///
class zkClass
{
    public:
	/** @name Zymkey Context
	 */
	///@{
        ///
        /// @brief Constructor: a Zymkey context is opened.
        /// @throws zkAppUtilsException upon error
        ///
        zkClass();
        ///
        /// @brief Destructor: the Zymkey context is closed
        ///
        virtual ~zkClass();
	///@}

        ///////////////////////////////////////////////////////////////////////
        // Random Number Generation
        ///////////////////////////////////////////////////////////////////////
        ///
	/** @name Random Number Generation
	 */
	///@{
        /// @brief Write random data to a file
        /// @param dst_filename
        ///        A string containing the absolute path to the file where random data is written
        /// @param rdata_sz
        ///        The number of random bytes to generate
        /// @throws zkAppUtilsException upon error
        ///
        void createRandDataFile(std::string& dst_filename, int rdata_sz);
        ///
        /// @brief Generate a block of random data
        /// @param rdata_sz
        ///     The number of random bytes to generate
        /// @returns A pointer to container with the random bytes
        /// @throws zkAppUtilsException upon error
        ///
        zkAppUtils::byteArray* getRandBytes(int rdata_sz);
	///@}

        ///////////////////////////////////////////////////////////////////////
        // Data locking
        ///////////////////////////////////////////////////////////////////////
        ///
	/** @name Lock Data
	 */
	///@{
        /// @brief Locks data from a plaintext source file and stores locked
        ///        data object to a destination file
        /// @param src_pt_filename
        ///        Absolute path to source plaintext file
        /// @param dst_ct_filename
        ///        Absolute path to destination file which will contain
        ///        locked data object
        /// @param use_shared_key
        ///        Determines whether one-way or shared key is used for locking.
        ///        Default = false.
        /// @throws zkAppUtilsException upon error
        ///
        void lockData(const std::string& src_pt_filename,
                      const std::string& dst_ct_filename,
                      bool use_shared_key = false);
        ///
        /// @brief Locks data from a plaintext source byte container and stores
        ///        locked data object to a destination file
        /// @param src_pt_data
        ///        Unsigned byte container which holds source plaintext data
        /// @param dst_ct_filename
        ///        Absolute path to destination file which will contain
        ///        locked data object
        /// @param use_shared_key
        ///        Determines whether one-way or shared key is used for locking.
        ///        Default = false.
        /// @throws zkAppUtilsException upon error
        ///
        void lockData(const byteArray& src_pt_data,
                      const std::string& dst_ct_filename,
                      bool use_shared_key = false);
        ///
        /// @brief Locks data from a plaintext source file and stores
        ///        locked data object in a container of unsigned bytes
        /// @param src_pt_filename
        ///        Absolute path to source plaintext file
        /// @param use_shared_key
        ///        Determines whether one-way or shared key is used for locking.
        ///        Default = false.
        /// @returns Byte container with locked data
        /// @throws zkAppUtilsException upon error
        ///
        zkAppUtils::byteArray* lockData(const std::string& src_pt_filename,
                                        bool use_shared_key = false);
        ///
        /// @brief Locks data from a plaintext source byte container and stores
        ///        locked data object in a container of unsigned bytes
        /// @param src_pt_data
        ///        Unsigned byte container which holds source plaintext data
        /// @param use_shared_key
        ///        Determines whether one-way or shared key is used for locking.
        ///        Default = false.
        /// @returns Byte container with locked data
        /// @throws zkAppUtilsException upon error
        ///
        zkAppUtils::byteArray* lockData(const byteArray& src_pt_data,
                                        bool use_shared_key = false);
	///@}

        ///////////////////////////////////////////////////////////////////////
        // Data unlocking
        ///////////////////////////////////////////////////////////////////////
	/** @name Unlock Data
         */
        ///@{
        /// @brief Unocks a locked data object from source file and stores
        ///        unlocked data object to a destination file
        /// @param src_ct_filename
        ///        Absolute path to source file which contains locked data
        ///        object
        /// @param dst_pt_filename
        ///        Absolute path to destination file which will contain
        ///        unlocked plaintext data
        /// @param use_shared_key
        ///        Determines whether one-way or shared key is used for locking.
        ///        Default = false.
        /// @throws zkAppUtilsException upon error
        ///
        void unlockData(const std::string& src_ct_filename,
                        const std::string& dst_pt_filename,
                        bool use_shared_key = false);
        ///
        /// @brief Unlocks a locked data object contained in an unsigned byte
        ///        container and stores plaintext data to a destination file
        /// @param src_ct_data
        ///        Unsigned byte container which holds locked data object
        /// @param dst_pt_filename
        ///        Absolute path to destination file which will contain
        ///        unlocked plaintext data
        /// @param use_shared_key
        ///        Determines whether one-way or shared key is used for locking.
        ///        Default = false.
        /// @throws zkAppUtilsException upon error
        ///
        void unlockData(const byteArray& src_ct_data,
                        const std::string& dst_pt_filename,
                        bool use_shared_key = false);
        ///
        /// @brief Unlocks a locked data object from a plaintext source file and stores
        ///        plaintext data in a container of unsigned bytes
        /// @param src_ct_filename
        ///        Absolute path to source file which contains locked data
        ///        object
        /// @param use_shared_key
        ///        Determines whether one-way or shared key is used for locking.
        ///        Default = false.
        /// @returns Byte container with plaintext data
        /// @throws zkAppUtilsException upon error
        ///
        zkAppUtils::byteArray* unlockData(const std::string& src_ct_filename,
                                          bool use_shared_key = false);
        ///
        /// @brief Unlocks a locked data object contained in an unsigned byte
        ///        container and stores plaintext data in a container of
        ///        unsigned bytes
        /// @param src_ct_data
        ///        Unsigned byte container which holds locked data object
        /// @param use_shared_key
        ///        Determines whether one-way or shared key is used for locking.
        ///        Default = false.
        /// @returns Byte container with plaintext data
        /// @throws zkAppUtilsException upon error
        ///
        zkAppUtils::byteArray* unlockData(const byteArray& src_ct_bytes,
                                          bool use_shared_key = false);
        ///@}

        ///////////////////////////////////////////////////////////////////////
        // ECDSA
        ///////////////////////////////////////////////////////////////////////
	/** @name ECDSA
         */
        ///@{
        /// @brief Generate a signature from a data digest using the Zymkey's
        ///        private key
        /// @param digest
        ///        Currently, this should be a SHA256 digest
        /// @param slot
        ///        The key slot to use for verification.
        /// @returns Byte container with binary signature
        /// @throws zkAppUtilsException upon error
        ///
        zkAppUtils::byteArray* genECDSASigFromDigest(byteArray& digest,
                                                     int slot = 0);
        /// @brief Generate a signature from a data digest using the Zymkey's
        ///        private key
        /// @param digest
        ///        Currently, this should be a SHA256 digest
        /// @param slot
        ///        The key slot to use for verification.
        /// @param recovery_id
        ///        [Output]Returns recovery id value needed for ethereum key recovery.
        /// @returns Byte container with binary signature
        /// @throws zkAppUtilsException upon error
        ///
        zkAppUtils::byteArray* genECDSASigFromDigest(byteArray& digest,
                                                     uint8_t& recovery_id,
                                                     int slot = 0);
        ///
        /// @brief Verify a signature from a data digest using the Zymkey's
        ///        public key. The public key is not given as an input. Rather,
        ///        the Zymkey uses its own copy of the private key. This
        ///        insures that the public key that matches the private key
        ///        is used.
        /// @param digest
        ///        Currently, this should be a SHA256 digest
        /// @param sig
        ///        The signature to verify
        /// @param pub_key_slot
        ///        The key slot to use for verification.
        /// @param pub_key_is_foreign
        ///        If false, the normal key store is referenced. Otherwise,
        ///        the foreign public key store is referenced.
        /// @param pub_key_is_foreign
        ///        If false, the normal key store is referenced. Otherwise,
        ///        the foreign public key store is referenced.
        /// @returns true = signature verification passed, false = signature verification failed
        bool verifyECDSASigFromDigest(byteArray& digest,
                                      byteArray& sig,
                                      int pub_key_slot = 0,
                                      bool pub_key_is_foreign = false);
        ///@}

        ///////////////////////////////////////////////////////////////////////
        // ECDH (Supported Devices: HSM6, Secure Compute Module)
        ///////////////////////////////////////////////////////////////////////
        /** @name ECDH and KDF
	 */
        ///@{
        /// @brief Perform a raw ECDH operation. (Supported Devices: HSM6, Secure Compute Module)
        /// @details Perform an ECDH operation with no Key Derivation Function (KDF). The
        ///          raw pre-master secret is returned in the response. The peer public
        ///          key is presented in the call.
        /// @param slot
        ///        The key slot to use for the local key. If this parameter is
        ///        -1, the ephemeral key is used.
        /// @param peer_pubkey
        ///        The peer public key.
        /// @returns Byte container with pre master secret
        zkAppUtils::byteArray* doRawECDH(int slot,
                                         byteArray& peer_pubkey);
        ///
        /// @brief Perform a raw ECDH operation. (Supported Devices: HSM6, Secure Compute Module)
        /// @details Perform an ECDH operation with no Key Derivation Function (KDF). The
        ///           raw pre-master secret is returned in the response. The peer public
        ///           key is referenced from the zymkey internal key store.
        /// @param slot
        ///        The key slot to use for the local key. If this parameter is
        ///                -1, the ephemeral key is used.
        /// @param peer_pubkey_slot
        ///        The peer public key slot where the peer public key is to be
        ///        found.
        /// @param peer_pubkey_slot_is_foreign
        ///        If true, the peer public key slot is found in the foreign
        ///        public keyring.
        /// @returns Byte container with pre master secret
        zkAppUtils::byteArray* doRawECDHWithIntPeerPubkey(int slot,
                                                          int peer_pubkey_slot,
                                                          bool peer_pubkey_slot_is_foreign);
       ///
       /// @brief Perform an ECDH operation plus Key Derivation Function. (Supported Devices: HSM6, Secure Compute Module)
       /// @details Perform an ECDH operation with Key Derivation Function (KDF). The
       ///          derived key is returned in the response. The peer public key is
       ///          presented in the call.
       ///  @param slot
       ///         The key slot to use for the local key. If this parameter is
       ///         -1, the ephemeral key is used.
       ///  @param peer_pubkey
       ///         The peer public key.
       /// @param salt
       ///        The salt to use for the selected KDF.
       /// @param info
       ///        The info field to use for RFC 5869. Ignored for PBKDF2.
       /// @param num_iterations
       ///        Number of iterations to carry out (PBKDF only)
       /// @param derived_key_sz
       ///        The desired number of bytes to return for the KDF. For
       ///        RFC 5869, this value must be less than 8160 bytes (SHA256) or
       ///        16320 (SHA512).
       /// @return Byte container with pre master secret.
       zkAppUtils::byteArray* doECDHAndKDF(ZK_ECDH_KDF_TYPE kdf_type,
                                           int slot,
                                           byteArray& peer_pubkey,
                                           byteArray& salt,
                                           byteArray& info,
                                           int num_iterations,
                                           int derived_key_sz);
       /// @brief Perform an ECDH operation plus Key Derivation Function. (Supported Devices: HSM6, Secure Compute Module)
       /// @details Perform an ECDH operation with Key Derivation Function (KDF). The
       ///          derived key is returned in the response. The peer public key is
       ///          referenced from the zymkey internal key store.
       /// @param slot
       ///        The key slot to use for the local key. If this parameter is
       ///        -1, the ephemeral key is used.
       /// @param peer_pubkey_slot
       ///        The peer public key slot where the peer public key is to be
       ///        found.
       /// @param peer_pubkey_slot_is_foreign
       ///        If true, the peer public key slot is found in the foreign
       ///        public keyring.
       /// @param salt
       ///        The salt to use for the selected KDF.
       /// @param info
       ///        The info field to use for RFC 5869. Ignored for PBKDF2.
       /// @param num_iterations
       ///        Number of iterations to carry out (PBKDF only)
       /// @param derived_key_sz
       ///        The desired number of bytes to return for the KDF. For
       ///        RFC 5869, this value must be less than 8160 bytes (SHA256) or
       ///        16320 (SHA512).
       /// @return Byte container with pre master secret.
       zkAppUtils::byteArray* doECDHAndKDFWithIntPeerPubkey(ZK_ECDH_KDF_TYPE kdf_type,
                                                            int slot,
                                                            int peer_pubkey_slot,
                                                            bool peer_pubkey_slot_is_foreign,
                                                            byteArray& salt,
                                                            byteArray& info,
                                                            int num_iterations,
                                                            int derived_key_sz);

        ///@}

	/** @name Key Management
         */
        ///@{

        ///////////////////////////////////////////////////////////////////////
        // Key management (Supported Devices: HSM6, Secure Compute Module)
        ///////////////////////////////////////////////////////////////////////
        ///
        /// @brief [DEPRECATED] Use exportPubKey2File. Save the public key that matches the Zymkey's private key into a
        ///        PEM formatted file. Mainly used for Certificate Signing Request
        ///        (CSR) generation.
        /// @param dst_filename
        ///        Absolute location where the PEM formatted file is to be stored
        /// @param slot
        ///        Reserved for future use
        /// @throws zkAppUtilsException upon error
        ///
        void saveECDSAPubKey2File(std::string dst_filename,
                                  int slot = 0);
        ///
        /// @brief Store the public key to a host file in PEM format.
        /// @details This function is useful for generating Certificate Signing Requests
        ///          (CSR).
        /// @param dst_filename
        ///        Filename where PEM formatted public key is to be stored.
        /// @param pubkey_slot
        ///        The key slot to retrieve. Zymkey and HSM4 have slots 0, 1, and 2.
        /// @param slot_is_foreign
        ///        If true, designates the pubkey slot to come from the foreign
        ///        keystore. (Supported Devices: HSM6, Secure Compute Module)
        /// @throws zkAppUtilsException upon error
        void exportPubKey2File(std::string dst_filename,
                               int pubkey_slot = 0,
                               bool slot_is_foreign = false);
        ///
        /// @brief [DEPRECATED] Use exportPubKey. Get a container of bytes which contains the ECDSA public key
        /// @param slot
        ///        Reserved for future use.
        /// @returns Byte container with binary public key
        /// @throws zkAppUtilsException upon error
        ///
        zkAppUtils::byteArray* getECDSAPubKey(int slot = 0);
        ///
        /// @brief Gets the public key and stores in a byte array created by this
        ///        function.
        /// @param pubkey_slot
        ///        The key slot to retrieve. Zymkey and HSM4 have slots 0, 1, and 2.
        /// @param slot_is_foreign
        ///        If true, designates the pubkey slot to come from the foreign
        ///        keystore. (Supported Devices: HSM6, Secure Compute Module)
        /// @throws zkAppUtilsException upon error
        zkAppUtils::byteArray* exportPubKey(int pubkey_slot = 0,
                                            bool slot_is_foreign = 0);

        ///
        /// @brief Get the list of allocated keys. (Supported Devices: HSM6, Secure Compute Module)
        /// @details This function returns a list of all allocated key slots.
        /// @param foreign_key_pool
        ///        (input) if true, retrieve allocation list of the foreign keys
       /// @returns Array of the allocated keys.
        intArray* getAllocSlotsList(bool foreign_key_pool);

        ///
        /// @brief Store a new foreign public key in Zymkey. (Supported Devices: HSM6, Secure Compute Module)
        /// @details This function stores a new foreign public key in the Zymkey
        ///          public key ring. This public key can be used for signature
        ///          verification in use cases where it is desirable to hide the public
        ///          key.
        /// @param pk_type
        ///        The type of the public key.
        ///        {ZK_NISTP256, ZK_SECP256R1 = ZK_NISTP256, ZK_SECP256K1}
        /// @param pk
        ///        Public key to store.
        /// @returns Allocated slot number in foreign key store, less than 0 for failure.
        /// @throws zkAppUtilsException upon error
        int storeForeignPubKey(ZK_EC_KEY_TYPE pk_type,
                                  byteArray& pk);
        ///
        /// @brief Prevent a public key from being exported to the host. (Supported Devices: HSM6, Secure Compute Module)
        /// @details This function prevents the public key at the specified slot from
        ///          being exported to the host using the API zkExportPubKey.
        /// @param pubkey_slot
        ///        The key slot to disable pubkey export on.
        /// @param slot_is_foreign
        ///        The slot parameter refers to a slot in the foreign keyring.
        /// @throws zkAppUtilsException upon error
        void disablePubKeyExport(int pubkey_slot,
                                 bool slot_is_foreign = false);
        ///
        /// @brief Generate a new persistent key pair. (Supported Devices: HSM6, Secure Compute Module)
        /// @details This function generates a new key pair of the specified type and
        ///          store it persistently. This key pair cannot be used as part of the
        ///          zymkey's digital wallet operations.
        /// @param type
        ///        The type of key to generate (ZK_EC_KEY_TYPE).
        ///        {ZK_NISTP256, ZK_SECP256R1 = ZK_NISTP256, ZK_SECP256K1}
        /// @returns Allocated slot number if successful, less than 0 for failure.
        /// @throws zkAppUtilsException upon error
        int genKeyPair(ZK_EC_KEY_TYPE type);
        ///
        /// @brief Generate an ephemeral key pair. (Supported Devices: HSM6, Secure Compute Module)
        /// @details This function generates an ephemeral key pair of the specified type.
        ///          Ephemeral key pairs are useful when performing ECDH for
        ///          time-of-flight encryption. Only one ephemeral key slot is available
        ///          and is not persistent between reboots.
        /// @param type
        ///        The type of key to generate (ZK_EC_KEY_TYPE).
        ///        {ZK_NISTP256, ZK_SECP256R1 = ZK_NISTP256, ZK_SECP256K1}
        /// @throws zkAppUtilsException upon error
        void genEphemeralKeyPair(ZK_EC_KEY_TYPE type);
        ///
        /// @brief Remove a key pair or a foreign public key. (Supported Devices: HSM6, Secure Compute Module)
        /// @details This function deletes a key pair or a foreign public key from
        ///          persistent storage.
        /// @param slot
        ///        The slot
        /// @param slot_is_foreign
        ///        The slot parameter refers to a slot in the foreign keyring.
        /// @throws zkAppUtilsException upon error
        void removeKey(int slot,
                      bool slot_is_foreign = false);
        ///
        /// @brief Invalidate the ephemeral key. (Supported Devices: HSM6, Secure Compute Module)
        /// @details This function invalidates the ephemeral key.
        /// @throws zkAppUtilsException upon error
        void invalidateEphemeralKey();

        ///@}

	/** @name Digital Wallet (BIP32/39/44)
         */
        ///@{
        ///////////////////////////////////////////////////////////////////////
        // Wallet BIP32/39/44
        ///////////////////////////////////////////////////////////////////////

        ///
        /// @brief Generate a BIP32 master seed to start a new blockchain wallet. (Supported Devices: HSM6, Secure Compute Module)
        /// @details This function generates a new blockchain master seed for
        ///          creating a new wallet per BIP32.
        /// @param type
        ///        The type of key to generate (ZK_EC_KEY_TYPE).
        ///        {ZK_NISTP256, ZK_SECP256R1 = ZK_NISTP256, ZK_SECP256K1, ZK_ED25519, ZK_X25519}
        /// @param wallet_name
        ///        An ASCII string which contains the name of the wallet.
        /// @param mnemonics(output) Allocated mnemonics if it was asked for, based on recovery strategy used.
        /// @param master_generator_key
        ///        The master generator key used to help generate the master
        ///        seed (bip32). Defaults to empty string
        /// @param recovery_strategy
        ///        The recovery strategy object for returning BIP39 or SLIP39 (starts a SLIP39 session instead of a one shot).
        ///        Also contains key variants and passphrases. Defaults to no recovery strategy.
        /// @returns The slot the master seed was allocated to.

        int genWalletMasterSeed(ZK_EC_KEY_TYPE type,
                                std::string& wallet_name,
                                const recoveryStrategyBaseType& recovery_strategy = recoveryStrategyBaseType(),
                                const byteArray& master_generator_key = byteArray());

        int genWalletMasterSeed(ZK_EC_KEY_TYPE type,
                                std::string& wallet_name,
                                const recoveryStrategyBIP39& recovery_strategy,
                                std::string* mnemonic_str,
                                const byteArray& master_generator_key = byteArray());

        int genWalletMasterSeed(ZK_EC_KEY_TYPE type,
                                std::string& wallet_name,
                                const recoveryStrategySLIP39& recovery_strategy,
                                const byteArray& master_generator_key = byteArray());

        ///
        /// @brief Set the active SLIP39 group and the amount of members needed. (Supported Devices: HSM6, Secure Compute Module)
        /// @details This function configures the active group to generate the number of shards
        ///          requested for the active group.
        /// @param group_index
        ///        The index of the group to generate shards from. Index starts at 0.
        /// @param member_count
        ///        The total amount of member shards in this group to generate.
        /// @param member_threshold
        ///        The number of member shards needed to recreate this group in recovery.
        /// @returns 0 if successful on configuring the active group, less than 0 for failure.
        int setGenSLIP39GroupInfo(int group_index,
                                  int member_count,
                                  int member_threshold);

        ///
        /// @brief Generate a new SLIP39 member shard. (Supported Devices: HSM6, Secure Compute Module)
        /// @details This function generates a new SLIP39 member shard.
        ///          The shard can optionally have a password attached to it.
        /// @param passphrase
        ///        Password for the shard. Can be empty string.
        /// @param mnemonic_str (output)
        ///        The mnemonic sentence of the shard.
        /// @returns 0 if successful on configuring the active group, less than 0 for failure.
        int addGenSLIP39Member(std::string passphrase,
                               std::string* mnemonic_str);

        ///
        /// @brief Cancels the current active SLIP39 session (Supported Devices: HSM6, Secure Compute Module)
        /// @details This function cancels open active SLIP39 sessions. For both
        ///          generation and restore SLIP39 sessions.
        /// @returns 0 if successful on configuring the active group, less than 0 for failure.
        int cancelSLIP39Session();

        ///
        /// @brief Generate a oversight wallet from the last hardened node of a wallet node address (Supported Devices: HSM6, Secure Compute Module)
        /// @details This function generates a new oversight wallet and are used to generate
        ///          public keys in a deterministic way.
        /// @param type
        ///        The type of key to generate (ZK_EC_KEY_TYPE).
        ///        {ZK_NISTP256, ZK_SECP256R1 = ZK_NISTP256, ZK_SECP256K1, ZK_ED25519, ZK_X25519}
        /// @param public_key
        ///        The public_key of the last hardened node of the address.
        /// @param chain_code
        ///        The chain_code of the last hardened node of the address.
        /// @param node_address
        ///        The node_address being derived from.
        /// @param wallet_name
        ///        An ASCII string which contains the name of the wallet.
        /// @param variant
        ///        The key type variant to generate.
        ///        Currently only "cardano" for ed25519 is supported.
        /// @returns The slot the oversight wallet was allocated to.
        int genOverSightWallet(ZK_EC_KEY_TYPE type,
                               byteArray& public_key,
                               byteArray& chain_code,
                               std::string& node_address,
                               std::string& wallet_name,
                               std::string& variant);
        ///
        /// @brief Generate child key from a parent key in a wallet (Supported Devices: HSM6, Secure Compute Module)
        /// @details This function generates a new child key descendent from a specified
        ///          parent key in a wallet.
        /// @param parent_key_slot
        ///        The parent key slot to base the child key derivation on.
        /// @param index
        ///        The index of the child seed. This determines the node address
        ///        as well as the outcome of the key generation.
        /// @param is_hardened
        ///        If true, a hardened key is generated.
        /// @param return_chain_code
        ///        If true, exports the chain code.
        ///        Must be exported from a hardened node.
        /// @param (output) Allocated chain_code if it was asked for.
        /// @returns Allocated slot number if successful, less than 0 for failure.
        /// @throws zkAppUtilsException upon error
        int genWalletChildKey(int parent_key_slot,
                              uint32_t index,
                              bool is_hardened,
                              bool return_chain_code,
                              byteArray* chain_code);
        ///
        /// @brief Restore a master seed from a BIP39 mnemonic and a master generator
        ///        key. (Supported Devices: HSM6, Secure Compute Module)
        /// @details This function restores a wallet master seed from a supplied BIP39
        ///          mnemonic string and a master generator key.
        /// @param type
        ///        The type of key to generate (ZK_KEY_TYPE).
        ///        {ZK_NISTP256, ZK_SECP256R1 = ZK_NISTP256, ZK_SECP256K1}
        /// @param wallet_name
        ///        An ASCII string which contains the name of the wallet.
        /// @param master_generator_key
        ///        The master generator key used to help generate the master
        ///        seed.
        /// @param recovery_strategy
        ///        The recovery strategy used.
        ///        Contains the passphrases and key variant to recover from.
        /// @param mnemonic_str
        ///        The mnemonic sentence to recover the master seed with. (Not used for SLIP39)
        /// @returns Allocated slot number if successful, less than 0 for failure.
        /// @throws zkAppUtilsException upon error
        int restoreWalletMasterSeedFromMnemonic(ZK_EC_KEY_TYPE type,
                                                std::string& wallet_name,
                                                byteArray& master_generator_key,
                                                recoveryStrategyBIP39& recovery_strategy,
                                                std::string& mnemonic_str);

        int restoreWalletMasterSeedFromMnemonic(ZK_EC_KEY_TYPE type,
                                                std::string& wallet_name,
                                                byteArray& master_generator_key,
                                                recoveryStrategySLIP39& recovery_strategy);

        ///
        /// @brief Feed a SLIP39 shard to restore a master seed (Supported Devices: HSM6, Secure Compute Module).
        /// @details This function will feed a shard to the module until the conditions
        ///          are met and a master seed is generated.
        /// @param passphrase
        ///        The passphrase that was attached to the shard.
        /// @param mnemonic_sentence (output)
        ///        The twenty-four word sentence mnemonic shard.
        /// @returns allocated slot number when all shards required are fed in, less than 0 for no change.
        int addRestoreSLIP39Mnemonic(std::string passphrase,
                                     std::string mnemonic_sentence);
        ///
        /// @brief Derive the node address from a key slot number. (Supported Devices: HSM6, Secure Compute Module)
        /// @details This function derives a node address from an input key slot number.
        /// @param slot
        ///        (input) A key slot number that is part of a digital wallet.
        /// @param out_node_addr
        ///        (output) A pointer to a string which will contain the node address in
        ///                 ASCII.
        /// @param wallet_name
        ///        (output) A pointer to a string which will contain the wallet name in
        ///                 ASCII. If NULL, this parameter will not be retrieved.
        /// @returns The master seed key slot.
        /// @throws zkAppUtilsException upon error
        int getWalletNodeAddrFromKeySlot(int slot,
                                         std::string* out_node_addr,
                                         std::string* out_wallet_name = NULL);
        ///
        /// @brief Derive the slot address from a node address. (Supported Devices: HSM6, Secure Compute Module)
        /// @details This function returns the slot number associated with a given node
        ///          address.
        /// @param node_addr
        ///        (input) A pointer which contains the node address in ASCII.
        /// @param wallet_name
        ///        (input) A pointer which contains the wallet name in ASCII, used to
        ///                identify the wallet identity. If desired, this parameter can
        ///                be NULL and the master_seed_slot parameter can be specified
        ///                instead.
        /// @param master_seed_slot
        ///        (input) The master seed slot. Can be used to specify the wallet
        ///                identity instead of the wallet name.
        /// @returns The associated key slot.
        /// @throws zkAppUtilsException upon error
        int getWalletKeySlotFromNodeAddr(std::string& node_addr,
                                         std::string& wallet_name,
                                         int master_seed_slot);

        ///@}

	/** @name LED Control
         */
        ///@{


        ///////////////////////////////////////////////////////////////////////
        // LED
        ///////////////////////////////////////////////////////////////////////
        ///
        /// @brief Turn LED off
        /// @throws zkAppUtilsException upon error
        ///
        void ledOff();
        ///
        /// @brief Turn LED on
        /// @throws zkAppUtilsException upon error
        ///
        void ledOn();
        ///
        /// @brief Flash LED a certain number of times
        /// @param on_ms
        ///        Number of milliseconds that the LED will be on during a flash cycle
        /// @param off_ms
        ///        Number of milliseconds that the LED will be off during a flash cycle
        /// @param num_flashes
        ///        Number of flash cycles to execute. 0 = infinite.
        /// @throws zkAppUtilsException upon error
        ///
        void ledFlash(uint32_t on_ms,
                      uint32_t off_ms = 0,
                      uint32_t num_flashes = 0);

        ///@}

	/** @name Administrative
         */
        ///@{
        ///////////////////////////////////////////////////////////////////////
        // Administrative
        ///////////////////////////////////////////////////////////////////////
        ///
        /// @brief Sets the i2c address (i2c versions only). Used in case of
        ///        i2c bus device address conflict.
        /// @param addr
        ///        The i2c address to set. Upon successful change, the Zymkey
        ///        will reset itself. However, if the address is the same as the
        ///        current setting, the Zymkey will not reset. Valid address
        ///        ranges are 0x30 - 0x37 and 0x60 - 0x67.
        /// @throws zkAppUtilsException upon error
        ///
        void setI2CAddr(int addr);

        ///@}

	/** @name Time
         */
        ///@{
        ///////////////////////////////////////////////////////////////////////
        // Time
        ///////////////////////////////////////////////////////////////////////
        ///
        /// @brief Get current GMT time
        /// @details This method is called to get the time directly from a
        ///          Zymkey's Real Time Clock (RTC)
        /// @param precise_time
        ///        (input) If true, this API returns the time after the next second
        ///        falls. This means that the caller could be blocked up to one second.
        ///        If false, the API returns immediately with the current time reading.
        /// @returns The time in seconds from the epoch (Jan. 1, 1970).
        /// @throws zkAppUtilsException upon error
        ///
        uint32_t getTime(bool precise_time = false);
        ///@}

	/** @name Accelerometer
         */
        ///@{

        ///////////////////////////////////////////////////////////////////////
        // Accelerometer
        ///////////////////////////////////////////////////////////////////////
        ///
        /// @brief Sets the sensitivity of the tap detection as a percentage
        ///        for an individual axis or all axes.
        /// @param pct
        ///        Sensitivity expressed in percentage. 0% = off, 100% = maximum.
        /// @param axis
        ///        X, Y, Z or all (default).
        /// @throws zkAppUtilsException upon error
        ///
        void setTapSensitivity(float pct,
                               ZK_ACCEL_AXIS_TYPE axis = ZK_ACCEL_AXIS_ALL);
        ///
        /// @brief Wait for a tap event to be detected
        /// @details This function is called in order to wait for a tap event to occur.
        ///          This function blocks the calling thread unless called with a
        ///          timeout of zero.
        /// @param timeout_ms
        ///        (input) The maximum amount of time in milliseconds to wait for a tap
        ///        event to arrive.
        /// @throws zkAppUtilsTimeoutException upon timeout or
        ///         zkAppUtilsException upon other errors
        ///
        void waitForTap(uint32_t timeout_ms);
        ///
        /// @brief Get current accelerometer data and tap info.
        /// @details This function gets the most recent accelerometer data in units of g
        ///          forces plus the tap direction per axis.
        ///          Array index 0 = x
        ///                      1 = y
        ///                      2 = z
        /// @param accel_data
        ///        (output) The current accelerometer data and tap information
        /// @throws zkAppUtilsException upon error
        ///
        void getAccelerometerData(accelData& accel_data);

        ///@}

	/** @name Binding Management
         */
        ///@{
        ///////////////////////////////////////////////////////////////////////
        // Binding management
        ///////////////////////////////////////////////////////////////////////
        ///
        /// @brief Set soft binding lock.
        /// @details This function locks the binding for a specific HSM. This API is
        ///          only valid for HSM series products.
        /// @throws zkAppUtilsException upon error
        ///
        void lockBinding();

        ///
        /// @brief Get current binding info
        /// @details This function gets the current binding lock state as well as the
        ///          current binding state. This API is only valid for devices in the HSM
        ///          family.
        /// @param binding_is_locked
        ///        (output) Binary value which expresses the current binding lock state.
         ///        is_bound
         ///        (output) Binary value which expresses the current bind state.
        /// @throws zkAppUtilsException upon error
        ///
        void getCurrentBindingInfo(bool& binding_is_locked, bool& is_bound);

        ///@}

	/** @name Perimeter Detect
         */
        ///@{
        ///////////////////////////////////////////////////////////////////////
        // Perimeter detect
        ///////////////////////////////////////////////////////////////////////
        ///
        /// @brief Set perimeter breach action
        /// @details This function specifies the action to take when a perimeter breach
        ///          event occurs. The possible actions are any combination of:
        ///              - Notify host
        ///              - Zymkey self-destruct
        /// @param channel
        ///        (input) The channel (0 or 1) that the action flags will be applied to.
        /// @param action_flags
        ///        (input) The actions to apply to the perimeter event channel:
        ///               - Notify (ZK_PERIMETER_EVENT_ACTION_NOTIFY)
        ///               - Self-destruct (ZK_PERIMETER_EVENT_ACTION_SELF_DESTRUCT)
        /// @throws zkAppUtilsException upon error
        ///
        void setPerimeterEventAction(int channel, uint32_t action_flags);
	///
	/// @brief Set the low power period (Supported Devices: HSM6, Secure Compute Module).
	/// @details This function sets low power period on the digital perimeter detect
	/// @param lp_period
	///        (input) low power period in microseconds
	/// @returns void.
	/// @throws zkAppUtilsException upon error
	void setDigitalPerimeterDetectLPPeriod(int lp_period);
	///
	/// @brief Set the low power max number of bits (Supported Devices: HSM6, Secure Compute Module).
	/// @details This function sets low power max number of bits on the digital perimeter detect
	/// @param max_num_bits
	///        (input) max number of bits
	/// @returns void.
	/// @throws zkAppUtilsException upon error
	void setDigitalPerimeterDetectLPMaxBits(int max_num_bits);

	///
	/// @brief Set the delays (Supported Devices: HSM6, Secure Compute Module).
	/// @details This function sets delays on the digital perimeter detect
	/// @param min_delay_ns
	///        (input) minimum delay in nanoseconds
	/// @param max_delay_ns
	///        (input) maximum delay in nanoseconds
	/// @returns void.
	/// @throws zkAppUtilsException upon error
	void setDigitalPerimeterDetectDelays(int min_delay_ns,
						      int max_delay_ns);

        ///
        /// @brief Wait for a perimeter breach event to be detected
        /// @details This function is called in order to wait for a perimeter breach
        ///          event to occur. This function blocks the calling thread unless
        ///          called with a timeout of zero. Note that, in order to receive
        ///          perimeter events, the zymkey must have been configured to notify
        ///          the host on either or both of the perimeter detect channels via a
        ///          call to "zkSetPerimeterEventAction".
        /// @param timeout_ms
        ///        (input) The maximum amount of time in milliseconds to wait for a
        ///        perimeter event to arrive.
        /// @throws zkAppUtilsException upon error
        ///
        void waitForPerimeterEvent(uint32_t timeout_ms);
        ///
        /// @brief Get current perimeter detect info.
        /// @details This function gets the timestamp of the first perimeter detect
        ///          event for the given channel
        /// @param timestamps_sec
        ///        (output) The timestamps for when any events occurred.
        ///                 The index in this array corresponds to the channel number used by zkSetPerimeterEventAction.
        ///                 A 0 value means no breach has occurred on this channel. This array is allocated by
        ///                 this routine and so it must be freed by the caller.
        /// @param num_timestamps
        ///         (output) The number of timestamps in the returned array
        /// @throws zkAppUtilsException upon error
        ///
        void getPerimeterDetectInfo(uint32_t** timestamp_sec, int& num_timestamps);
        ///
        /// @brief Clear perimeter detect info.
        /// @details This function clears all perimeter detect info and rearms all
        ///          perimeter detect channels
        /// @throws zkAppUtilsException upon error
        ///
        void clearPerimeterDetectEvents();

	/// @brief Get Zymkey model number
	/// @details This function retrieves the model number of the zymkey referred to in a specified context
	/// @returns A pointer to a string containing the Zymkey model number.
	/// @throws zkAppUtilsException upon error
	std::string* getModelNumberString();

	/// @brief Get Zymkey firmware version.
	/// @details This function retrieves the firmware version of the zymkey referred to in a specified context.
	/// @returns A pointer to a string containing the Zymkey firmware version.
	/// @throws zkAppUtilsException upon error
	std::string* getFirmwareVersionString();

	/// @brief Get Zymkey serial number.
	/// @details This function retrieves the serial number of the zymkey referred to in a specified context.
	/// @returns A pointer to a string containing the Zymkey serial number.
	/// @throws zkAppUtilsException upon error
	std::string* getSerialNumberString();


	///@}

	/** @name Module Info
         */
        ///@{
	/// @brief Get the CPU Temp. (Supported Devices: HSM6, Secure Compute Module)
	/// @details This function gets the current HSM CPU temp.
	/// @returns The CPU temp as a float.
	/// @throws zkAppUtilsException upon error
   float getCPUTemp();

   ///
   /// @brief Get the AUX Temp. (Secure Compute Modules Only)
   /// @details This function gets the current temp of an aux processor. (defaults to 0). THIS FUNCTION IS FOR INTERNAL ZYMBIT USE ONLY.
   /// @returns The CPU temp as a float.
   /// @throws zkAppUtilsException upon error
   float getAUXTemp(int index = 0);

	/// @brief Get the RTC drift. (Supported Devices: HSM6, Secure Compute Module)
	/// @details This function gets the current RTC drift.
	/// @returns The RTC drift as a float.
	/// @throws zkAppUtilsException upon error
   float getRTCDrift();

	/// @brief Get the battery voltage(Supported Devices: HSM6, Secure Compute Module).
	/// @details This function gets the current battery voltage
	/// @returns the battery voltage as a float.
	/// @throws zkAppUtilsException upon error
	float getBatteryVoltage();
        ///@}

	/** @name Battery Voltage Monitor
         */
        ///@{

  /**
   * @brief Set battery voltage threshold action. (Supported Devices: HSM6, Secure Compute Module)
   * @details This function specifies the action to take when the
   *          battery voltage falls below the threshold set by
   *          zkSetBatteryVoltageThreshold. If this function is never
   *          called, do nothing is default. There are three actions:
   *              - Do nothing
   *              - Go to sleep until battery is replaced
   *              - Self-destruct
   * @param action
   *        (input) The action to apply, specify one of the
   *                ZK_THRESHOLD_ACTION_TYPE:
   *                - Do nothing (ZK_ACTION_NONE)
   *                - Sleep (ZK_ACTION_SLEEP)
   *                - Self-destruct (ZK_ACTION_SELF_DESTRUCT)
   * @throws zkAppUtilsException upon error
   */
  void setBatteryVoltageAction(ZK_THRESHOLD_ACTION_TYPE action);

  /**
   * @brief Sets the battery voltage threshold. (Supported Devices: HSM6, Secure Compute Module)
   * @details This function sets the threshold at which if the
   *          battery voltage falls bellow, the action set by
   *          zkSetBatteryVoltageAction will be carried out. The
   *          recommended threshold is 2.3V is assumed by default. Threshold
   *          must be below 2.5V.
   * @param threshold
   *        (input) The threshold in Volts.
   * @throws zkAppUtilsException upon error
   */
  void setBatteryVoltageThreshold(float threshold);

  ///@}

	/** @name CPU Temperature Monitor
   */
  ///@{

  /**
   * @brief Set HSM CPU temperature threshold action. (Supported Devices: HSM6, Secure Compute Module)
   * @details This function specifies the action to take when the
   *          HSM CPU temperature falls below the threshold set by
   *          zkSetCPULowTempThreshold, or rises above the threshold
   *          set by zkSetCPUHighTempThreshold. There are two actions
   *          to apply:
   *              - Do nothing
   *              - Self-destruct
   * @param action
   *        (input) The action to apply, used it's named
   *                constant from ZK_THRESHOLD_ACTION_TYPE:
   *                - Do nothing (ZK_ACTION_NONE)
   *                - Self-destruct (ZK_ACTION_SELF_DESTRUCT)
   * @throws zkAppUtilsException upon error
   */
  void setCPUTempAction(ZK_THRESHOLD_ACTION_TYPE action);

  /**
   * @brief Sets the HSM CPU low temperature threshold. (Supported Devices: HSM6, Secure Compute Module)
   * @details This function sets the threshold at which if the
   *          on-board HSM CPU's tempreature falls below, the
   *          action set by zkSetCPUTempAction will be carried out.
   *          WARNING: You can lock yourself out in dev mode if
   *          you set a threshold above the CPU's ambient temperature.
   *          The recommended setting is no more than 20C.
   *          If no threshold is set, -10 degrees Celsius is set as
   *          default.
   * @param threshold
   *        (input) The threshold in celsius.
   * @throws zkAppUtilsException upon error
   */
  void setCPULowTempThreshold(float threshold);

  /**
   * @brief Sets the HSM CPU high temperature threshold. (Supported Devices: HSM6, Secure Compute Module)
   * @details This function sets the threshold at which if the
   *          on-board HSM CPU's tempreature rises above, the
   *          action set by zkSetCPUTempAction will be carried out.
   *          WARNING: You can lock yourself out in dev mode if
   *          you set a threshold below the CPU's ambient temperature.
   *          The recommended setting is no less than 40C.
   *          If no threshold is set, 65 degrees celsius is set as
   *          default.
   * @param threshold
   *        (input) The threshold in celsius.
   * @return 0 for success, less than 0 for failure.
   */
  void setCPUHighTempThreshold(float threshold);

  /**
   * @brief Sets the Supervised boot policy. (Supported Devices: Secure Compute Module)
   * @details This function specifies the action to take when Supervised boot
   *          event triggers based on a file change.
   * @param policy_id
   *        (input) The actions to apply to the Supervised boot process:
   *                - 0 Do Nothing (ZK_SUPBOOT_FAIL_NO_ACTION)
   *                - 1 Self-Destruct (ZK_SUPBOOT_FAIL_DESTROY)
   *                - 2 Hold Chip in Reset (ZK_SUPBOOT_FAIL_HOLD_RESET)
   * @return 0 for success, less than 0 for failure.
   */
  void setSupervisedBootPolicy(int policy_id);

  /**
   * @brief Update file manifest for Supervised boot to check. (Supported Devices: Secure Compute Module)
   * @details This function adds or updates files to be checked by Supervised boot.
   * @param file_path
   *        (input) The file to be signed and checked by Supervised boot.
   * @param slot
   *        (input) The slot to sign the file with. Defaults to 15, if no slot 15, defaults to 0.
   * @return 0 for success, less than 0 for failure.
   */
  void addOrUpdateSupervisedBootFile(std::string file_path, int slot = 15);

  /**
   * @brief Remove a file from file manifest for Supervised boot to check. (Supported Devices: Secure Compute Module)
   * @details This function removes a file to be checked by Supervised boot.
   * @param file_path
   *        (input) The file to be removed from the Supervised boot manifest.
   * @return 0 for success, less than 0 for failure.
   */
  void removeSupervisedBootFile(std::string file_path);

  /**
   * @brief Get file manifest for Supervised boot to check. (Supported Devices: Secure Compute Module)
   * @details This function gets the list of files to be checked by Supervised boot.
   * @param manifest
   *        (output) The file manifest that is checked by Supervised boot.
   * @return a string for the list of files in the manifest.
   */
  std::string* getSupervisedBootFileManifest();
  ///@}


    protected:
    private:
        ///
        /// @var Zymkey context. Created by constructor.
        ///
        zkCTX zkContext;
};

}; // namespace zkAppUtils

#endif // ZKAPPUTILSCLASS_H

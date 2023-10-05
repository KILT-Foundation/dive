/**
 * @file zk_app_utils.h
 * @author Scott Miller
 * @version 1.0
 * @date November 17, 2016
 * @copyright Zymbit, Inc.
 * @brief C interface to Zymkey Application Utilities Library.
 * @details
 * This file contains the C API to the the Zymkey Application Utilities
 * library. This API facilitates writing user space applications which use
 * Zymkey to perform cryptographic operations, such as:
 *      1. Signing of payloads using ECDSA
 *      2. Verification of payloads that were signed using Zymkey
 *      3. Exporting the public key that matches Zymkey's private key
 *      4. "Locking" and "unlocking" data objects
 *      5. Generating random data
 * Additionally, there are functions for changing the i2c address (i2c units
 * only), setting tap sensitivity and controlling the LED.
 */

#ifndef __ZK_APP_UTILS_H
#define __ZK_APP_UTILS_H

#ifdef __cplusplus
extern "C"
{
#endif // __cplusplus

#include <stdbool.h>
#include <stdint.h>

/**
 * @typedef The typedef for the Zymkey Context type.
 */
typedef void* zkCTX;

/**
 * @brief Supported key types
 */
typedef enum ZK_EC_KEY_TYPE
{
    ZK_NISTP256,
    ZK_SECP256R1 = ZK_NISTP256,
    ZK_SECP256K1,
    ZK_ED25519,
    ZK_X25519
} ZK_EC_KEY_TYPE;

/**
 * @brief Supported recovery strategies
 */
typedef enum ZK_RECOVERY_STRATEGY
{
    ZK_NONE,
    ZK_BIP39,
    ZK_SLIP39
} ZK_RECOVERY_STRATEGY;

/**
 * @brief Supported ECDH key derivation function types
 */
typedef enum ZK_ECDH_KDF_TYPE
{
    ZK_KDF_RFC5869_SHA256,
    ZK_KDF_RFC5869_SHA512,
    ZK_KDF_PBKDF2_SHA256,
    ZK_KDF_PBKDF2_SHA512,
} ZK_ECDH_KDF_TYPE;

/**
 * @brief Accelerometer axis enum, used to set tap sensitivity.
 */
typedef enum ZK_ACCEL_AXIS_TYPE
{
    ZK_ACCEL_AXIS_X,
    ZK_ACCEL_AXIS_Y,
    ZK_ACCEL_AXIS_Z,
    ZK_ACCEL_AXIS_ALL
} ZK_ACCEL_AXIS_TYPE;

/**
 * @brief Possible actions for threshold monitor functions
 */
typedef enum ZK_THRESHOLD_ACTION_TYPE
{
    ZK_ACTION_NONE,
    ZK_ACTION_SELF_DESTRUCT,
    ZK_ACTION_SLEEP
} ZK_THRESHOLD_ACTION_TYPE;


/**
 * @brief zkGetAccelerometer data output.
 *
 */
typedef struct zkAccelAxisDataType
{
    double g;           /**< the axis reading in units of g-force */
    int tapDirection;   /**< the direction of the force along the axis which caused a tap event:
                          * -1 = negative
                          * +1 = positive
                          * 0  = did not cause a tap event
                          */
} zkAccelAxisDataType;

/**
 * @brief Perimeter breach action flag definitions.
 */
#define ZK_PERIMETER_EVENT_ACTION_NOTIFY        (1 << 0)
#define ZK_PERIMETER_EVENT_ACTION_SELF_DESTRUCT (1 << 1)

/** @name Zymkey Context
 */
/**@{*/
/**
 * @brief Open a Zymkey context.
 * @param ctx
 *        (output) returns a pointer to a Zymkey context.
 * @return 0 for success, less than 0 for failure.
 */
int zkOpen(zkCTX* ctx);

/**
 * @brief Close a Zymkey context.
 * @param ctx
 *        (input) The Zymkey context to close
 * @return 0 for success, less than 0 for failure.
 */
int zkClose(zkCTX ctx);
/**@}*/


/** @name Random Number Generation
 */
/**@{*/
/**
 * @brief Fill a file with random numbers.
 * @param ctx
 *        (input) Zymkey context.
 * @param dst_filename
 *        (input) Absolute path name for the destination file.
 * @param rdata_sz
 *        (input) The number of random bytes to generate.
 * @return 0 for success, less than 0 for failure.
 */
int zkCreateRandDataFile(zkCTX ctx, const char* dst_filename, int rdata_sz);

/**
 * @brief Get an array of random bytes.
 * @param ctx
 *        (input) Zymkey context.
 * @param rdata
 *        (input) Pointer to a pointer of bytes.
 * @param rdata_sz
 *        (input) The number of random bytes to generate.
 * @return 0 for success, less than 0 for failure.
 */
int zkGetRandBytes(zkCTX ctx, uint8_t** rdata, int rdata_sz);
/**@}*/

/**  @name Lock Data
 */
/**@{*/
/**
 * @brief Lock up source (plaintext) data from a file and store the results
 * (ciphertext) in a destination file
 * @details
 *   This function encrypts and signs a block of plaintext data from a file
 *   and stores the result in a destination file.
 * @note
 *   The zymkey has two keys that can be used for locking/unlocking
 *   operations, designated as 'shared' and 'one-way'.
 *     1. The one-way key is meant to lock up data only on the
 *        local host computer. Data encrypted using this key cannot
 *        be exported and deciphered anywhere else.
 *     2. The shared key is meant for publishing data to other
 *        sources that have the capability to generate the shared
 *        key, such as the Zymbit cloud server.
 *
 * @param ctx
 *        (input) Zymkey context.
 * @param src_pt_filename
 *        (input) The absolute path to the file where the source (plaintext) data
 *        is located.
 * @param dst_ct_filename
 *        (input) The absolute path to the file where the destination (ciphertext)
 *        data should be deposited.
 * @param use_shared_key
 *        (input) This parameter specifies which key will be used to
 *        used to lock the data up. A value of 'false' specifies that the Zymkey
 *        will use the one-way key whereas 'true' specifies that the shared key
 *        will be used. Specify 'true' for publishing data to another  that has the
 *        shared key (e.g. Zymbit cloud) and 'False' when the data is meant to
 *        reside exclusively withing the host computer.
 * @return 0 for success, less than 0 for failure.
 */
int zkLockDataF2F(zkCTX ctx,
                  const char* src_pt_filename,
                  const char* dst_ct_filename,
                  bool use_shared_key);

/**
 * @brief Lock up source (plaintext) data from a byte array and store the results
 * (ciphertext) in a destination file
 * @details
 *   This function encrypts and signs a block of binary plaintext data
 *   and stores the result in a destination file.
 * @note (See zkLockDataF2F for notes about keys)
 *
 * @param ctx
 *        (input) Zymkey context.
 * @param src_pt
 *        (input) Binary plaintext source byte array.
 * @param src_pt_sz
 *        (input) Size of plaintext source data.
 * @param dst_ct_filename
 *        (input) The absolute path to the file where the destination (ciphertext)
 *        data should be deposited.
 * @param use_shared_key
 *        (input) Specifies if shared key is to be used. See zkLockDataF2F.
 * @return 0 for success, less than 0 for failure.
 */
int zkLockDataB2F(zkCTX ctx,
                  const uint8_t* src_pt,
                  int src_pt_sz,
                  const char* dst_ct_filename,
                  bool use_shared_key);

/**
 * @brief Lock up source (plaintext) data from a file and store the results
 * (ciphertext) in a destination byte array.
 * @details
 *   This function encrypts and signs a block of plaintext data from a file
 *   and stores the result in a binary byte array.
 * @note (See zkLockDataF2F for notes about keys)
 *
 * @param ctx
 *        (input) Zymkey context.
 * @param src_pt_filename
 *        (input) The absolute path to the file where the source (plaintext) data
 *        is located.
 * @param dst_ct
 *        (output) A pointer to a pointer to an array of unsigned bytes created by
 *        this function. This pointer must be freed by the application when no longer
 *        needed.
 * @param dst_ct_sz
 *        (output) A pointer to an integer which contains the size of the
 *        destination array.
 * @param use_shared_key
 *        (input) Specifies if shared key is to be used. See zkLockDataF2F.
 * @return 0 for success, less than 0 for failure.
 */
int zkLockDataF2B(zkCTX ctx,
                  const char* src_pt_filename,
                  uint8_t** dst_ct,
                  int* dst_ct_sz,
                  bool use_shared_key);

/**
 * @brief Lock up source (plaintext) data from a byte array and store the results
 * (ciphertext) in a destination byte array.
 * @details
 *   This function encrypts and signs a block of plaintext data and stores the
 *   result in a binary byte array.
 * @note (See zkLockDataF2F for notes about keys)
 *
 * @param ctx
 *        (input) Zymkey context.
 * @param src_pt
 *        (input) Binary plaintext source byte array.
 * @param src_pt_sz
 *        (input) Size of plaintext source data.
 * @param dst_ct
 *        (output) A pointer to a pointer to an array of unsigned bytes created by
 *        this function. This pointer must be freed by the application when no longer
 *        needed.
 * @param dst_ct_sz
 *        (output) A pointer to an integer which contains the size of the
 *        destination array.
 * @param use_shared_key
 *        (input) Specifies if shared key is to be used. See zkLockDataF2F.
 * @return 0 for success, less than 0 for failure.
 */
int zkLockDataB2B(zkCTX ctx,
                  const uint8_t* src_pt,
                  int src_pt_sz,
                  uint8_t** dst_ct,
                  int* dst_ct_sz,
                  bool use_shared_key);

/**@}*/

/** @name Unlock Data
 */
/**@{*/
/**
 * @brief Unlock source (ciphertext) data from a file and store the results
 * (plaintext) in a destination file
 * @details
 *   This function verifies a locked object signature and decrypts the
 *   associated ciphertext data.
 * @note (See zkLockDataF2F for notes about keys)
 *
 * @param ctx
 *        (input) Zymkey context.
 * @param src_ct_filename
 *        (input) The absolute path to the file where the source (ciphertext) data
 *        is located.
 * @param dst_pt_filename
 *        (input) The absolute path to the file where the destination (plaintext)
 *        data should be deposited.
 * @param use_shared_key
 *        (input) This parameter specifies which key will be used to
 *        used to lock the data up. A value of 'false' specifies that the Zymkey
 *        will use the one-way key whereas 'true' specifies that the shared key
 *        will be used. Specify 'true' for publishing data to another  that has the
 *        shared key (e.g. Zymbit cloud) and 'False' when the data is meant to
 *        reside exclusively withing the host computer.
 * @return 0 for success, less than 0 for failure.
 */
int zkUnlockDataF2F(zkCTX ctx,
                    const char* src_ct_filename,
                    const char* dst_pt_filename,
                    bool use_shared_key);

/**
 * @brief Unlock source (ciphertext) data from a byte array and store the results
 * (plaintext) in a destination file
 * @details
 *   This function verifies a locked object signature and decrypts the
 *   associated ciphertext data.
 * @note (See zkLockDataF2F for notes about keys)
 *
 * @param ctx
 *        (input) Zymkey context.
 * @param src_ct
 *        (input) Binary ciphertext source byte array.
 * @param src_ct_sz
 *        (input) Size of ciphertext source data.
 * @param dst_pt_filename
 *        (input) The absolute path to the file where the destination (plaintext)
 *        data should be deposited.
 * @param use_shared_key
 *        (input) Specifies if shared key is to be used. See zkLockDataF2F.
 * @return 0 for success, less than 0 for failure.
 */
int zkUnlockDataB2F(zkCTX ctx,
                    const uint8_t* src_ct,
                    int src_ct_sz,
                    const char* dst_pt_filename,
                    bool use_shared_key);

/**
 * @brief Unlock source (ciphertext) data from a file and store the results
 * (plaintext) in a destination byte array.
 * @details
 *   This function verifies a locked object signature and decrypts the
 *   associated ciphertext data.
 * @note (See zkLockDataF2F for notes about keys)
 *
 * @param ctx
 *        (input) Zymkey context.
 * @param src_ct_filename
 *        (input) The absolute path to the file where the source (ciphertext) data
 *        is located.
 * @param dst_pt
 *        (output) A pointer to a pointer to an array of unsigned bytes created by
 *        this function. This pointer must be freed by the application when no longer
 *        needed.
 * @param dst_pt_sz
 *        (output) A pointer to an integer which contains the size of the
 *        destination array.
 * @param use_shared_key
 *        (input) Specifies if shared key is to be used. See zkLockDataF2F.
 * @return 0 for success, less than 0 for failure.
 */
int zkUnlockDataF2B(zkCTX ctx,
                    const char* src_ct_filename,
                    uint8_t** dst_pt,
                    int* dst_pt_sz,
                    bool use_shared_key);

/**
 * @brief Unlock source (ciphertext) data from a byte array and store the results
 * (plaintext) in a destination byte array.
 * @details
 *   This function verifies a locked object signature and decrypts the
 *   associated ciphertext data.
 * @note (See zkLockDataF2F for notes about keys)
 *
 * @param ctx
 *        (input) Zymkey context.
 * @param src_ct
 *        (input) Binary ciphertext source byte array.
 * @param src_ct_sz
 *        (input) Size of ciphertext source data.
 * @param dst_pt
 *        (output) A pointer to a pointer to an array of unsigned bytes created by
 *        this function. This pointer must be freed by the application when no longer
 *        needed.
 * @param dst_pt_sz
 *        (output) A pointer to an integer which contains the size of the
 *        destination array.
 * @param use_shared_key
 *        (input) Specifies if shared key is to be used. See zkLockDataF2F.
 * @return 0 for success, less than 0 for failure.
 */
int zkUnlockDataB2B(zkCTX ctx,
                    const uint8_t* src_ct,
                    int src_ct_sz,
                    uint8_t** dst_pt,
                    int* dst_pt_sz,
                    bool use_shared_key);

/**@}*/

/** @name ECDSA
 */
/**@{*/

/**
 * @brief Generate a signature using the Zymkey's ECDSA private key.
 * @param ctx
 *        (input) Zymkey context.
 * @param digest
 *        (input) This parameter contains the digest of the data that
 *        will be used to generate the signature.
 * @param slot
 *        (input) The key slot to generate a signature from. This parameter is
 *        only valid for Zymkey models 4i and beyond.
 * @param sig
 *        (output) A pointer to a pointer to an array of unsigned bytes which
 *        contains the generated signature. This pointer is created by this function
 *        and must be freed by the application when no longer needed.
 * @param sig_sz
 *        (output) A pointer to an integer which contains the size of the signature.
 * @return 0 for success, less than 0 for failure.
 */
int zkGenECDSASigFromDigest(zkCTX ctx,
                            const uint8_t* digest,
                            int slot,
                            uint8_t** sig,
                            int* sig_sz);

/**
* @brief Generate a signature using the Zymkey's ECDSA private key.
* @param ctx
*        (input) Zymkey context.
* @param digest
*        (input) This parameter contains the digest of the data that
*        will be used to generate the signature.
* @param slot
*        (input) The key slot to generate a signature from. This parameter is
*        only valid for Zymkey models 4i and beyond.
* @param sig
*        (output) A pointer to a pointer to an array of unsigned bytes which
*        contains the generated signature. This pointer is created by this function
*        and must be freed by the application when no longer needed.
* @param sig_sz
*        (output) A pointer to an integer which contains the size of the signature.
* @param recovery_id
*        (output) Returns recovery id value needed for ethereum key recovery.
* @return 0 for success, less than 0 for failure.
*/
int zkGenECDSASigFromDigestWithRecID(zkCTX ctx,
                            const uint8_t* digest,
                            int slot,
                            uint8_t** sig,
                            int* sig_sz,
                            uint8_t* recovery_id);

/**
 * @brief Verify a signature using one of the Zymkey's public keys.
 * @details Verify a signature using an internal public key from the Zymkey
 *          private/public key store.
 * @param ctx
 *        (input) Zymkey context.
 * @param digest
 *        (input) This parameter contains the digest of the data that
 *        will be used to generate the signature.
 * @param pubkey_slot
 *        (input) The key slot to generate a signature from. This parameter is
 *        only valid for Zymkey models 4i and beyond.
 * @param sig
 *        (input) Array of bytes which contains the signature.
 * @param sig_sz
 *        (input) Size of signature.
 * @return 0 for signature verification failed, 1 for signature verification
 *         passed, less than 0 for general failure.
 */
int zkVerifyECDSASigFromDigest(zkCTX ctx,
                               const uint8_t* digest,
                               int pubkey_slot,
                               const uint8_t* sig,
                               int sig_sz);

/**
 * @brief Verify a signature using one of the Zymkey's foreign public keys.
 * @details Verify a signature using a public key from the Zymkey foreign key
 *          store.
 * @param ctx
 *        (input) Zymkey context.
 * @param digest
 *        (input) This parameter contains the digest of the data that
 *        will be used to generate the signature.
 * @param pubkey_slot
 *        (input) The key slot to generate a signature from. This parameter is
 *        only valid for Zymkey models 4i and beyond.
 * @param sig
 *        (input) Array of bytes which contains the signature.
 * @param sig_sz
 *        (input) Size of signature.
 * @return 0 for signature verification failed, 1 for signature verification
 *         passed, less than 0 for general failure.
 */
int zkVerifyECDSASigFromDigestWithForeignKeySlot(zkCTX ctx,
                                                 const uint8_t* digest,
                                                 int pubkey_slot,
                                                 const uint8_t* sig,
                                                 int sig_sz);
/**@}*/

/** @name ECDH and KDF
 */
/**@{*/
/**
 * @brief Perform a raw ECDH operation. (Supported Devices: HSM6, Secure Compute Module)
 * @details Perform an ECDH operation with no Key Derivation Function (KDF). The
 *          raw pre-master secret is returned in the response. The peer public
 *          key is presented in the call.
 * @param ctx
 *        (input) Zymkey context.
 * @param slot
 *        (input) The key slot to use for the local key. If this parameter is
 *                -1, the ephemeral key is used.
 * @param peer_pubkey
 *        (input) The peer public key.
 * @param peer_pubkey_sz
 *        (input) Size of the peer public key.
 * @param pre_master_secret
 *        (output) returned pointer to the pre-master secret
 * @return 0 for success, less than 0 for general failure.
 */
int zkDoRawECDH(zkCTX ctx,
                int slot,
                const uint8_t* peer_pubkey,
                int peer_pubkey_sz,
                uint8_t** pre_master_secret);

/**
 * @brief Perform a raw ECDH operation. (Supported Devices: HSM6, Secure Compute Module)
 * @details Perform an ECDH operation with no Key Derivation Function (KDF). The
 *          raw pre-master secret is returned in the response. The peer public
 *          key is referenced from the zymkey internal key store.
 * @param ctx
 *        (input) Zymkey context.
 * @param slot
 *        (input) The key slot to use for the local key. If this parameter is
 *                -1, the ephemeral key is used.
 * @param peer_pubkey_slot
 *        (input) The peer public key slot where the peer public key is to be
 *                found.
 * @param peer_pubkey_slot_is_foreign
 *        (input) If true, the peer public key slot is found in the foreign
 *                public keyring.
 * @param pre_master_secret
 *        (output) returned pointer to the pre-master secret
 * @return 0 for success, less than 0 for general failure.
 */
int zkDoRawECDHWithIntPeerPubkey(zkCTX ctx,
                                 int slot,
                                 int peer_pubkey_slot,
                                 bool peer_pubkey_slot_is_foreign,
                                 uint8_t** pre_master_secret);

/**
 * @brief Perform an ECDH operation plus Key Derivation Function. (Supported Devices: HSM6, Secure Compute Module)
 * @details Perform an ECDH operation with Key Derivation Function (KDF). The
 *          derived key is returned in the response. The peer public key is
 *          presented in the call.
 * @param ctx
 *        (input) Zymkey context.
 * @param slot
 *        (input) The key slot to use for the local key. If this parameter is
 *                -1, the ephemeral key is used.
 * @param peer_pubkey
 *        (input) The peer public key.
 * @param peer_pubkey_sz
 *        (input) Size of the peer public key.
 * @param salt
 *        (input) The salt to use for the selected KDF.
 * @param salt_sz
 *        (input) The salt size. Must be less than or equal to 128 bytes.
 * @param info
 *        (input) The info field to use for RFC 5869. Ignored for PBKDF2.
 * @param info_sz
 *        (input) The size of the info parameter. Must be less than or equal to
 *                128 bytes.
 * @param num_iterations
 *        (input) Number of iterations to carry out (PBKDF only)
 * @param derived_key_sz
 *        (input) The desired number of bytes to return for the KDF. For
 *                RFC 5869, this value must be less than 8160 bytes (SHA256) or
 *                16320 (SHA512).
 * @param derived_key
 *        (output) returned pointer to the derived key.
 * @return 0 for success, less than 0 for general failure.
 */
int zkDoECDHAndKDF(zkCTX ctx,
                   ZK_ECDH_KDF_TYPE kdf_type,
                   int slot,
                   const uint8_t* peer_pubkey,
                   int peer_pubkey_sz,
                   const uint8_t* salt,
                   int salt_sz,
                   const uint8_t* info,
                   int info_sz,
                   int num_iterations,
                   int derived_key_sz,
                   uint8_t** derived_key);

/**
 * @brief Perform an ECDH operation plus Key Derivation Function. (Supported Devices: HSM6, Secure Compute Module)
 * @details Perform an ECDH operation with Key Derivation Function (KDF). The
 *          derived key is returned in the response. The peer public key is
 *          referenced from the zymkey internal key store.
 * @param ctx
 *        (input) Zymkey context.
 * @param slot
 *        (input) The key slot to use for the local key. If this parameter is
 *                -1, the ephemeral key is used.
 * @param peer_pubkey_slot
 *        (input) The peer public key slot where the peer public key is to be
 *                found.
 * @param peer_pubkey_slot_is_foreign
 *        (input) If true, the peer public key slot is found in the foreign
 *                public keyring.
 * @param salt
 *        (input) The salt to use for the selected KDF.
 * @param salt_sz
 *        (input) The salt size. Must be less than or equal to 128 bytes.
 * @param info
 *        (input) The info field to use for RFC 5869. Ignored for PBKDF2.
 * @param info_sz
 *        (input) The size of the info parameter. Must be less than or equal to
 *                128 bytes.
 * @param num_iterations
 *        (input) Number of iterations to carry out (PBKDF only)
 * @param derived_key_sz
 *        (input) The desired number of bytes to return for the KDF. For
 *                RFC 5869, this value must be less than 8160 bytes (SHA256) or
 *                16320 (SHA512).
 * @param derived_key
 *        (output) returned pointer to the derived key.
 * @return 0 for success, less than 0 for general failure.
 */
int zkDoECDHAndKDFWithIntPeerPubkey(zkCTX ctx,
                                    ZK_ECDH_KDF_TYPE kdf_type,
                                    int slot,
                                    int peer_pubkey_slot,
                                    bool peer_pubkey_slot_is_foreign,
                                    const uint8_t* salt,
                                    int salt_sz,
                                    const uint8_t* info,
                                    int info_sz,
                                    int num_iterations,
                                    int derived_key_sz,
                                    uint8_t** derived_key);

/**@}*/
/** @name Key Management
 */
/**@{*/

/**
 * @brief [DEPRECATED] Use zkExportPubKey2File. Store the public key to a host file in PEM format.
 * @details This function is useful for generating Certificate Signing Requests
 *          (CSR).
 * @param ctx
 *        (input) Zymkey context.
 * @param filename
 *        (input) Filename where PEM formatted public key is to be stored.
 * @param slot
 *        (input) The key slot to retrieve. Only valid for model 4i and above.
 * @return 0 for success, less than 0 for failure.
 */
int zkSaveECDSAPubKey2File(zkCTX ctx,
                           const char* filename,
                           int slot);
/**
 * @brief Store the public key to a host file in PEM format.
 * @details This function is useful for generating Certificate Signing Requests
 *          (CSR).
 * @param ctx
 *        (input) Zymkey context.
 * @param filename
 *        (input) Filename where PEM formatted public key is to be stored.
 * @param pubkey_slot
 *        (input) The key slot to retrieve. Zymkey and HSM4 have slots 0, 1, and 2.
 * @param slot_is_foreign
 *        (input) If true, designates the pubkey slot to come from the foreign
 *                keystore. (Supported Devices: HSM6, Secure Compute Module)
 * @return 0 for success, less than 0 for failure.
 */
int zkExportPubKey2File(zkCTX ctx,
                        const char* filename,
                        int pubkey_slot,
                        bool slot_is_foreign);

/**
 * @brief [DEPRECATED] Use zkExportPubKey. Gets the public key and stores in a byte array created by
 *        this function.
 * @param ctx
 *        (input) Zymkey context.
 * @param pk
 *        (output) Pointer to a pointer created by this function which contains
 *        the public key.
 * @param pk_sz
 *        (output) Pointer to an integer which contains the size of the public
 *        key.
 * @param slot
 *        (input) The key slot to retrieve. Only valid for model 4i and above.
 * @return 0 for success, less than 0 for failure.
 */
int zkGetECDSAPubKey(zkCTX ctx,
                     uint8_t** pk,
                     int* pk_sz,
                     int slot);
/**
 * @brief Gets the public key and stores in a byte array created by this
 *        function.
 * @param ctx
 *        (input) Zymkey context.
 * @param pk
 *        (output) Pointer to a pointer created by this function which contains
 *        the public key.
 * @param pk_sz
 *        (output) Pointer to an integer which contains the size of the public
 *        key.
 * @param pubkey_slot
 *        (input) The key slot to retrieve. Zymkey and HSM4 have slots 0, 1, and 2.
 * @param slot_is_foreign
 *        (input) If true, designates the pubkey slot to come from the foreign
 *                keystore (Supported Devices: HSM6, Secure Compute Module).
 * @return 0 for success, less than 0 for failure.
 */
int zkExportPubKey(zkCTX ctx,
                  uint8_t** pk,
                  int* pk_sz,
                  int pubkey_slot,
                  bool slot_is_foreign);

/**
 * @brief Get the list of allocated keys (Supported Devices: HSM6, Secure Compute Module).
 * @details This function returns a list of all allocated key slots.
 * @param ctx
 *        (input) Zymkey context.
 * @param is_foreign
 *        (input) if true, retrieve allocation list of the foreign keys
 * @param max_num_keys
 *        (input) retrieves the key pool size
 * @param alloc_key_list
 *        (output) a pointer to an array of integers provided by this
 *                 function to the caller
 * @param alloc_key_list_sz
 *        (output) a pointer to an integer which contains the size of
 *                 the returned key list
 * @return 0 if successful, less than 0 for failure.
 */
int zkGetAllocSlotsList(zkCTX ctx,
                        bool is_foreign,
                        int* max_num_keys,
                        int** alloc_key_list,
                        int * alloc_key_list_sz);

/**
 * @brief Store a new foreign public key in Zymkey. (Supported Devices: HSM6, Secure Compute Module)
 * @details This function stores a new foreign public key in the Zymkey
 *          public key ring. This public key can be used for signature
 *          verification in use cases where it is desirable to hide the public
 *          key.
 * @param ctx
 *        (input) Zymkey context.
 * @param pk_type
 *        (input) The type of the public key.
 * @param pk
 *        (input) Pointer to the public key to store.
 * @param pk_sz
 *        (input) The public key size.
 * @return allocated slot number in foreign key store, less than 0 for failure.
 */
int zkStoreForeignPubKey(zkCTX ctx,
                         ZK_EC_KEY_TYPE pk_type,
                         uint8_t* pk,
                         int pk_sz);

/**
 * @brief Prevent a public key from being exported to the host. (Supported Devices: HSM6, Secure Compute Module)
 * @details This function prevents the public key at the specified slot from
 *          being exported to the host using the API zkExportPubKey.
 * @param ctx
 *        (input) Zymkey context.
 * @param pubkey_slot
 *        (input) The key slot to disable pubkey export on.
 * @param slot_is_foreign
 *        (input) The slot parameter refers to a slot in the foreign keyring.
 * @return 0 for success, less than 0 for failure.
 */
int zkDisablePubKeyExport(zkCTX ctx,
                          int pubkey_slot,
                          bool slot_is_foreign);

/**
 * @brief Generate a new persistent key pair. (Supported Devices: HSM6, Secure Compute Module)
 * @details This function generates a new key pair of the specified type and
 *          store it persistently. This key pair cannot be used as part of the
 *          zymkey's digital wallet operations.
 * @param ctx
 *        (input) Zymkey context.
 * @param type
 *        (input) The type of key to generate (ZK_EC_KEY_TYPE).
 * @return allocated slot number if successful, less than 0 for failure.
 */
int zkGenKeyPair(zkCTX ctx,
                 ZK_EC_KEY_TYPE type);

/**
 * @brief Generate an ephemeral key pair. (Supported Devices: HSM6, Secure Compute Module)
 * @details This function generates an ephemeral key pair of the specified type.
 *          Ephemeral key pairs are useful when performing ECDH for
 *          time-of-flight encryption. Only one ephemeral key slot is available
 *          and is not persistent between reboots.
 * @param ctx
 *        (input) Zymkey context.
 * @param type
 *        (input) The type of key to generate (ZK_EC_KEY_TYPE).
 * @return 0 if successful, less than 0 for failure.
 */
int zkGenEphemeralKeyPair(zkCTX ctx,
                          ZK_EC_KEY_TYPE type);

/**
 * @brief Remove a key pair or a foreign public key. (Supported Devices: HSM6, Secure Compute Module)
 * @details This function deletes a key pair or a foreign public key from
 *          persistent storage.
 * @param ctx
 *        (input) Zymkey context.
 * @param slot
 *        (input) The slot
 * @param slot_is_foreign
 *        (input) The slot parameter refers to a slot in the foreign keyring.
 * @return 0 if successful, less than 0 for failure.
 */
int zkRemoveKey(zkCTX ctx,
                int slot,
                bool slot_is_foreign);

/**
 * @brief Invalidate the ephemeral key. (Supported Devices: HSM6, Secure Compute Module)
 * @details This function invalidates the ephemeral key.
 * @param ctx
 *        (input) Zymkey context.
 * @return 0 for success, less than 0 for failure.
 */
int zkInvalidateEphemeralKey(zkCTX ctx);
/**@}*/

/** @name Digital Wallet (BIP32/39/44)
 */
/**@{*/

/**
* @brief Generate master seed to start a new blockchain wallet. (Supported Devices: HSM6, Secure Compute Module)
* @details This function generates a new blockchain master seed for
*          creating a new wallet.
* @param ctx
*        (input) Zymkey context.
* @param type
*        (input) The type of key to generate (ZK_EC_KEY_TYPE).
* @param variant
*        (input) The variant of the key_type to generate.
*                Currently only "cardano" is supported for ed25519.
* @param wallet_name
*        (input) An ASCII string which contains the name of the wallet.
* @param master_generator_key
*        (input) The master generator key used to help generate the master
*                seed. Can be empty string.
* @param master_generator_key_size
*        (input) The size of the master generator key. If 0, no master
*                generator key is used in the formulation of the master seed.
* @param passphrase
*        (input) The passphrase to generate a different BIP39_mnemonic.
*                Can be empty string.
* @param mnemonic
*        (output) A pointer to the mnemonic sentence. If NULL, the master
*                 seed is generated per BIP32. Otherwise, the master seed is
*                 generated per recovery strategy and the mnemonic sentence is returned in
*                 this parameter. The string is null terminated and encoded in
*                 UTF-8 NFKD from the English dictionary.
* @return allocated slot number if successful, less than 0 for failure.
*/
int zkGenWalletMasterSeedWithBIP39(zkCTX ctx,
                                   ZK_EC_KEY_TYPE type,
                                   const char* variant,
                                   const char* wallet_name,
                                   const uint8_t* master_generator_key,
                                   int master_generator_key_size,
                                   const char* passphrase,
                                   char** mnemonic);

/**
* @brief Generate master seed to start a new blockchain wallet. (Supported Devices: HSM6, Secure Compute Module)
* @details This function opens a session to generate a new blockchain master seed
*          with the ability to recover from SLIP39 shards.
* @param ctx
*        (input) Zymkey context.
* @param type
*        (input) The type of key to generate (ZK_EC_KEY_TYPE).
* @param variant
*        (input) The variant of the key_type to generate.
*                Currently only "cardano" is supported for ed25519.
* @param wallet_name
*        (input) An ASCII string which contains the name of the wallet.
* @param master_generator_key
*        (input) The master generator key used to help generate the master
*                seed. Can be empty string.
* @param master_generator_key_size
*        (input) The size of the master generator key. If 0, no master
*                generator key is used in the formulation of the master seed.
* @param group_count
*        (input) The total count of groups(shards) to split into.
* @param group_threshold
*        (input) The count of groups(shards) needed to restore the master seed.
* @param group_iteration_exponent
*        (input) The iteration of exponent of SLIP39.
* @param master_passphrase
*        (input) The master passphrase used for slip 39 recovery process.
* @return 0 if successful on opening a SLIP39 session, less than 0 for failure.
*/
int zkGenWalletMasterSeedWithSLIP39(zkCTX ctx,
                                    ZK_EC_KEY_TYPE type,
                                    const char* variant,
                                    const char* wallet_name,
                                    const uint8_t* master_generator_key,
                                    int master_generator_key_size,
                                    int group_count,
                                    int group_threshold,
                                    int group_iteration_exponent,
                                    const char* master_passphrase);

/**
* @brief Set the active SLIP39 group and the amount of members needed. (Supported Devices: HSM6, Secure Compute Module)
* @details This function configures the active group to generate the number of shards
*          requested for the active group.
* @param ctx
*        (input) Zymkey context.
* @param group_index
*        (input) The index of the group to generate shards from. Index starts at 0.
* @param member_count
*        (input) The total amount of member shards in this group to generate.
* @param member_threshold
*        (input) The number of member shards needed to recreate this group in recovery.
* @return 0 if successful on configuring the active group, less than 0 for failure.
*/
int zkSetSLIP39GroupInfo(zkCTX ctx,
                         int group_index,
                         int member_count,
                         int member_threshold);

/**
* @brief Generate a new SLIP39 member shard. (Supported Devices: HSM6, Secure Compute Module)
* @details This function generates a new SLIP39 member shard.
*          The shard can optionally have a password attached to it.
* @param ctx
*        (input) Zymkey context.
* @param passhrase
*        (input) Password for the shard. Can be empty string.
* @param mnemonic_sentence
*        (output) The mnemonic sentence of the shard.
* @return 0 if successful on opening a SLIP39 session, less than 0 for failure.
*/
int zkAddSLIP39MemberPassword(zkCTX ctx,
                              const char* passhrase,
                              char** mnemonic_sentence);

/**
* @brief Cancels the current active SLIP39 session (Supported Devices: HSM6, Secure Compute Module)
* @details This function cancels open active SLIP39 sessions. For both
*          generation and restore SLIP39 sessions.
* @param ctx
*        (input) Zymkey context.
* @return 0 if successful on aborting a SLIP39 session, less than 0 for failure.
*/
int zkCancelSLIP39Session(zkCTX ctx);

/**
* @brief Generate master seed to start a new blockchain wallet. (Supported Devices: HSM6, Secure Compute Module)
* @details This function generates a new blockchain master seed for
*          creating a new wallet.
* @param ctx
*        (input) Zymkey context.
* @param type
*        (input) The type of key to generate (ZK_EC_KEY_TYPE).
* @param pub_key
*        (input) The public key to create the oversight wallet from.
*                Should come from a hardened node in a node tree.
* @param chain_code
*        (input) The chain code of the public key being used to create the oversight wallet.
* @param node_addr
*        (input) The node address index of the public key being used.
* @param wallet_name
*        (input) The name of the oversight wallet being created.
* @return allocated slot number if successful, less than 0 for failure.
*/
int zkGenOversightWallet(zkCTX ctx,
                         ZK_EC_KEY_TYPE type,
                         const char* variant,
                         const uint8_t* pub_key,
                         const uint8_t* chain_code,
                         const char* node_addr,
                         const char* wallet_name);

/**
* @brief Generate child key from a parent key in a blockchain wallet
*       . (Supported Devices: HSM6, Secure Compute Module)
* @details This function generates a new child key descendent from a specified
*          parent key in a wallet.
* @param ctx
*        (input) Zymkey context.
* @param parent_key_slot
*        (input) The parent key slot to base the child key derivation on.
* @param index
*        (input) The index of the child seed. This determines the node address
*                as well as the outcome of the key generation.
* @param is_hardened
*        (input) If true, a hardened key is generated.
* @param return_chain_code
*        (input) If true, returns the chain code of the public key that was just generated.
                 Has to be a hardened node as well.
* @param chain_code
*        (output) the chain code of the public key
* @return allocated slot number if successful, less than 0 for failure.
*/
int zkGenWalletChildKey(zkCTX ctx,
                        int parent_key_slot,
                        uint32_t index,
                        bool is_hardened,
                        bool return_chain_code,
                        uint8_t** chain_code);

/**
 * @brief Restore a master seed from a BIP39 mnemonic and a master generator
 *        key. (Supported Devices: HSM6, Secure Compute Module)
 * @details This function restores a wallet master seed from a supplied BIP39
 *          mnemonic string and a master generator key.
 * @param ctx
 *        (input) Zymkey context.
 * @param type
 *        (input) The type of key to generate (ZK_KEY_TYPE).
 * @param variant
 *        (input) The variant of the key_type to generate.
 *                Currently only "cardano" is supported for ed25519.
 * @param wallet_name
 *        (input) An ASCII string which contains the name of the wallet.
 * @param master_generator_key
 *        (input) The master generator key used to help generate the master
 *                seed.
 * @param master_generator_key_size
 *        (input) The size of the master generator key. If 0, no master
 *                generator key is used in the formulation of the master seed.
 * @param BIP39_passphrase
 *        (input) The passphrase used to generate the BIP39_mnemonic.
 * @param BIP39_mnemonic
 *        (input) The BIP39_mnemonic string, null terminated and UTF-8 NFKD
 *                encoded from the BIP39 English dictionary.
 * @return allocated slot number if successful, less than 0 for failure.
 */
int zkRestoreWalletMasterSeedFromBIP39Mnemonic(zkCTX ctx,
                                               ZK_EC_KEY_TYPE type,
                                               const char* variant,
                                               const char* wallet_name,
                                               const uint8_t* master_generator_key,
                                               int master_generator_key_size,
                                               const char* BIP39_passphrase,
                                               char* BIP39_mnemonic);

/**
* @brief Open a SLIP39 restore master seed session (Supported Devices: HSM6, Secure Compute Module).
* @details This function starts a restore SLIP39 session, in order to start feeding
*          shards into.
* @param ctx
*        (input) Zymkey context.
* @param type
*        (input) The type of key to generate (ZK_KEY_TYPE).
* @param variant
*        (input) The variant of the key_type to generate.
*                Currently only "cardano" is supported for ed25519.
* @param wallet_name
*        (input) An ASCII string which contains the name of the wallet.
* @param master_generator_key
*        (input) The master generator key used to help generate the master
*                seed.
* @param master_generator_key_size
*        (input) The size of the master generator key. If 0, no master
*                generator key is used in the formulation of the master seed.
* @param SLIP39_passphrase
*        (input) The master passphrase.
* @return allocated slot number if successful, less than 0 for failure.
*/
int zkRestoreWalletMasterSeedFromSLIP39(zkCTX ctx,
                                        ZK_EC_KEY_TYPE type,
                                        const char* variant,
                                        const char* wallet_name,
                                        const uint8_t* master_generator_key,
                                        int master_generator_key_size,
                                        const char* SLIP39_passphrase);

/**
* @brief Feed a SLIP39 shard to restore a master seed (Supported Devices: HSM6, Secure Compute Module).
* @details This function will feed a shard to the module until the conditions
*          are met and a master seed is generated.
* @param ctx
*        (input) Zymkey context.
* @param passphrase
*        (input) The passphrase that was attached to the shard.
* @param mnemonic_sentence
*        (input) The twenty-four word sentence mnemonic shard.
* @return allocated slot number when all shards required are fed in, less than 0 for no change.
*/
int zkAddRestoreSLIP39Mnemonic(zkCTX ctx,
                              const char* passphrase,
                              const char* mnemonic_sentence);

/**
 * @brief Derive the node address from a key slot number. (Supported Devices: HSM6, Secure Compute Module)
 * @details This function derives a node address from an input key slot number.
 * @param ctx
 *        (input) Zymkey context.
 * @param slot
 *        (input) A key slot number that is part of a digital wallet.
 * @param node_addr
 *        (output) A pointer to a pointer which will contain the node address in
 *                 ASCII.
 * @param wallet_name
 *        (output) A pointer to a pointer which will contain the wallet name in
 *                 ASCII. If NULL, this parameter will not be retrieved.
 * @param master_seed_slot
 *        (output) A pointer to an integer which will contain the master seed
 *                 key slot. If NULL, this parameter will not be retrieved.
 * @return 0 if successful, less than 0 for failure.
 */
int zkGetWalletNodeAddrFromKeySlot(zkCTX ctx,
                                   int slot,
                                   char** node_addr,
                                   char** wallet_name,
                                   int* master_seed_slot);

/**
 * @brief Derive the slot address from a node address. (Supported Devices: HSM6, Secure Compute Module)
 * @details This function returns the slot number associated with a given node
 *          address.
 * @param ctx
 *        (input) Zymkey context.
 * @param node_addr
 *        (input) A pointer which contains the node address in ASCII.
 * @param wallet_name
 *        (input) A pointer which contains the wallet name in ASCII, used to
 *                identify the wallet identity. If desired, this parameter can
 *                be NULL and the master_seed_slot parameter can be specified
 *                instead.
 * @param master_seed_slot
 *        (input) The master seed slot. Can be used to specify the wallet
 *                identity instead of the wallet name.
 * @param slot
 *        (output) A pointer to an integer which contains the associated key
 *                 slot.
 * @return 0 if successful, less than 0 for failure.
 */
int zkGetWalletKeySlotFromNodeAddr(zkCTX ctx,
                                   const char* node_addr,
                                   const char* wallet_name,
                                   int master_seed_slot,
                                   int* slot);
/**@}*/

/** @name Perimeter Detect
 */
/**@{*/

/**
 * @brief Set perimeter breach action
 * @details This function specifies the action to take when a perimeter breach
 *          event occurs. The possible actions are any combination of:
 *              - Notify host
 *              - Zymkey self-destruct
 * @param channel
 *        (input) The channel (0 or 1) that the action flags will be applied to.
 * @param action_flags
 *        (input) The actions to apply to the perimeter event channel:
 *                - Notify (ZK_PERIMETER_EVENT_ACTION_NOTIFY)
 *                - Self-destruct (ZK_PERIMETER_EVENT_ACTION_SELF_DESTRUCT)
 * @return 0 for success, less than 0 for failure.
 */
int zkSetPerimeterEventAction(zkCTX ctx, int channel, uint32_t action_flags);

/**
* @brief Set the low power period (Supported Devices: HSM6, Secure Compute Module).
* @details This function sets low power period on the digital perimeter detect
* @param ctx
*        (input) Zymkey context.
* @param lp_period
*        (input) low power period in microseconds
* @return 0 if successful, less than 0 for failure.
*/
int zkSetDigitalPerimeterDetectLPPeriod(zkCTX ctx,
                                       int lp_period);

/**
 * @brief Set the low power max number of bits (Supported Devices: HSM6, Secure Compute Module).
 * @details This function sets low power max number of bits on the digital perimeter detect
 * @param ctx
 *        (input) Zymkey context.
 * @param max_num_bits
 *        (input) max number of bits
 * @return 0 if successful, less than 0 for failure.
 */
int zkSetDigitalPerimeterDetectLPMaxBits(zkCTX ctx,
                                         int max_num_bits);

/**
* @brief Set the delays (Supported Devices: HSM6, Secure Compute Module).
* @details This function sets delays on the digital perimeter detect
* @param ctx
*        (input) Zymkey context.
* @param min_delay_ns
*        (input) minimum delay in nanoseconds
* @param max_delay_ns
*        (input) maximum delay in nanoseconds
* @return 0 if successful, less than 0 for failure.
*/
int zkSetDigitalPerimeterDetectDelays(zkCTX ctx,
                                     int min_delay_ns,
                                     int max_delay_ns);
/**
 * @brief Wait for a perimeter breach event to be detected
 * @details This function is called in order to wait for a perimeter breach
 *          event to occur. This function blocks the calling thread unless
 *          called with a timeout of zero. Note that, in order to receive
 *          perimeter events, the zymkey must have been configured to notify
 *          the host on either or both of the perimeter detect channels via a
 *          call to "zkSetPerimeterEventAction".
 * @param timeout_ms
 *        (input) The maximum amount of time in milliseconds to wait for a
 *        perimeter event to arrive.
 * @return 0 for success, less than 0 for failure, -ETIMEDOUT when no perimeter
 *         events detected within the specified timeout
 */
int zkWaitForPerimeterEvent(zkCTX ctx, uint32_t timeout_ms);

/**
 * @brief Get current perimeter detect info.
 * @details This function gets the timestamp of the first perimeter detect
 *          event for the given channel
 * @param timestamps_sec
 *        (output) The timestamps for when any breach occurred.
 *                 The index in this array corresponds to the channel number used by zkSetPerimeterEventAction.
 *                 A 0 value means no breach has occurred on this channel. This array is allocated by
 *                 this routine and so it must be freed by the caller.
 * @param  num_timestamps
 *         (output) The number of timestamps in the returned array
 * @return 0 for success, less than 0 for failure.
 */
int zkGetPerimeterDetectInfo(zkCTX ctx, uint32_t** timestamps_sec, int* num_timestamps);

/**
 * @brief Clear perimeter detect events.
 * @details This function clears all perimeter detect event info and rearms all
 *          perimeter detect channels
 * @return 0 for success, less than 0 for failure.
 */
int zkClearPerimeterDetectEvents(zkCTX ctx);
/**@}*/

/** @name LED Control
 */
/**@{*/

/**
 * @brief Turns the LED off.
 * @param ctx
 *        (input) Zymkey context.
 * @return 0 for success, less than 0 for failure.
 */
int zkLEDOff(zkCTX ctx);

/**
 * @brief Turns the LED on.
 * @param ctx
 *        (input) Zymkey context.
 * @return 0 for success, less than 0 for failure.
 */
int zkLEDOn(zkCTX ctx);

/**
 * @brief Flashes the LED.
 * @param ctx
 *        (input) Zymkey context.
 * @param on_ms
 *        (input) The amount of time, in milliseconds, that the LED will stay
 *        on during a flash cycle.
 * @param off_ms
 *        (input) The amount of time, in milliseconds, that the LED will stay
 *        off during a flash cycle.
 * @param num_flashes
 *        (input) The number of on/off flash cycles to complete. If this
 *        parameter is 0, then the LED will flash indefinitely.
 * @return 0 for success, less than 0 for failure.
 */
int zkLEDFlash(zkCTX ctx,
               uint32_t on_ms,
               uint32_t off_ms,
               uint32_t num_flashes);
/**@}*/

/** @name Administrative Ops
 */
/**@{*/

/**
 * @brief Sets the i2c address of the Zymkey (i2c versions only)
 * @details This method should be called if the i2c address of the
 *    Zymkey is shared with another i2c device on the same i2c bus.
 *    The default i2c address for Zymkey units is 0x30. Currently,
 *    the address may be set in the ranges of 0x30 - 0x37 and
 *    0x60 - 0x67.
 *    After successful completion of this command, the Zymkey will
 *    reset itself.
 * @param addr
 *        (input) The i2c address that the Zymkey will set itself to.
 * @return 0 for success, less than 0 for failure.
 */
int zkSetI2CAddr(zkCTX ctx, int addr);
/**@}*/

/** @name Time
 */
/**@{*/
/**
 * @brief Get current GMT time
 * @details This function is called to get the time directly from a
 *          Zymkey's Real Time Clock (RTC)
 * @param epoch_time_sec
 *        (output) The time in seconds from the epoch (Jan. 1, 1970).
 * @param precise_time
 *        (input) If true, this API returns the time after the next second
 *        falls. This means that the caller could be blocked up to one second.
 *        If false, the API returns immediately with the current time reading.
 * @return 0 for success, less than 0 for failure.
 */
int zkGetTime(zkCTX ctx, uint32_t* epoch_time_sec, bool precise_time);
/**@}*/

/** @name Accelerometer
 */
/**@{*/

/**
 * @brief Sets the sensitivity of tap operations.
 *  @details This method permits setting the sensitivity of the tap
 *           detection feature. Each axis may be individually
 *           configured or all at once.
 * @param axis
 *        (input) The axis to configure. This parameter should contain
 *        one of the values in the enum typedef ACCEL_AXIS_TYPE.
 * @param pct
 *        (input) The sensitivity expressed as percentage.
 *        1. 0% = Shut down: Tap detection should not occur along the
 *           axis.
 *        2. 100% = Maximum sensitivity.
 * @return 0 for success, less than 0 for failure.
 */
int zkSetTapSensitivity(zkCTX ctx, int axis, float pct);

/**
 * @brief Wait for a tap event to be detected
 * @details This function is called in order to wait for a tap event to occur.
 *          This function blocks the calling thread unless called with a
 *          timeout of zero.
 * @param timeout_ms
 *        (input) The maximum amount of time in milliseconds to wait for a tap
 *        event to arrive.
 * @return 0 for success, less than 0 for failure, -ETIMEDOUT when no tap
 *         events detected within the specified timeout
 */
int zkWaitForTap(zkCTX ctx, uint32_t timeout_ms);

/**
 * @brief Get current accelerometer data and tap info.
 * @details This function gets the most recent accelerometer data in units of g
 *          forces plus the tap direction per axis.
 * @param x
 *        (output) x axis accelerometer information
 *        y
 *        (output) y axis accelerometer information
 *        z
 *        (output) z axis accelerometer information
 * @return 0 for success, less than 0 for failure.
 */
int zkGetAccelerometerData(zkCTX ctx, zkAccelAxisDataType* x, zkAccelAxisDataType* y, zkAccelAxisDataType* z);
/**@}*/

/** @name Binding Management
 */
/**@{*/

/**
 * @brief Set soft binding lock.
 * @details This function locks the binding for a specific HSM. This API is
 *          only valid for HSM series products.
 * @return 0 for success, less than 0 for failure.
 */
int zkLockBinding(zkCTX ctx);

/**
 * @brief Get current binding info
 * @details This function gets the current binding lock state as well as the
 *          current binding state. This API is only valid for devices in the HSM
 *          family.
 * @param binding_is_locked
 *        (output) Binary value which expresses the current binding lock state.
 *        is_bound
 *        (output) Binary value which expresses the current bind state.
 * @return 0 for success, less than 0 for failure.
 */
int zkGetCurrentBindingInfo(zkCTX ctx, bool* binding_is_locked, bool* is_bound);
/**@}*/

/** @name Module Info
 */
/**@{*/
/**
 * @brief Get Zymkey model number
 * @details This function retrieves the model number of the zymkey referred to
 *          in a specified context
 * @param ctx
 *        (input) Zymkey context which was created with zkOpen
 * @param model_str
 *        (output) A double pointer to the model string. This function allocates
 *                 this string. It is the caller's responsibility to free it.
 * @return 0 for success, less than 0 for failure.
 */
int zkGetModelNumberString(zkCTX ctx, char** model_str);

/**
 * @brief Get Zymkey firmware version
 * @details This function retrieves the firmware version number of the zymkey
 *          referred to in a specified context
 * @param ctx
 *        (input) Zymkey context which was created with zkOpen
 * @param version_str
 *        (output) A double pointer to the firmware version string. This
 *                 function allocates this string. It is the caller's
 *                 responsibility to free it.
 * @return 0 for success, less than 0 for failure.
 */
int zkGetFirmwareVersionString(zkCTX ctx, char** fw_ver_str);

/**
 * @brief Get Zymkey serial number
 * @details This function retrieves the serial number of the zymkey
 *          referred to in a specified context
 * @param ctx
 *        (input) Zymkey context which was created with zkOpen
 * @param serial_num_str
 *        (output) A double pointer to the serial number string. This
 *                 function allocates this string. It is the caller's
 *                 responsibility to free it.
 * @return 0 for success, less than 0 for failure.
 */
int zkGetSerialNumberString(zkCTX ctx, char** serial_num_str);

/**
 * @brief Get current HSM CPU temperature. (Supported Devices: HSM6, Secure Compute Module)
 * @details This function gets the current HSM CPU temp.
 * @param cpu_temp
 *        (output) The temperature in celsius of the CPU.
 * @return 0 for success, less than 0 for failure.
 */
int zkGetCPUTemp(zkCTX ctx, float* cpu_temp);

/**
 * @brief Get current AUX temperature. (Only for secure compute modules)
 * @details This function gets an aux temp. THIS FUNCTION IS FOR INTERNAL ZYMBIT USE ONLY.
 * @param ctx
 *        (input) Zymkey context.
 * @param index
 *        (input) Index for which aux temp to be polled.
 * @param aux_temp
 *        (output) The temperature in celsius.
 * @return 0 for success, less than 0 for failure.
 */
int zkGetAUXTemp(zkCTX ctx, int index, float* aux_temp);

/**
 * @brief Get current RTC drift. (Supported Devices: HSM6, Secure Compute Module)
 * @details This function is called to get the current RTC drift.
 * @param rtc_drift
 *        (output) The RTC drift.
 * @return 0 for success, less than 0 for failure.
 */
int zkGetRTCDrift(zkCTX ctx, float* rtc_drift);

/**
 * @brief Get the battery voltage (Supported Devices: HSM6, Secure Compute Module).
 * @details This function gets the current battery voltage
 * @param ctx
 *        (input) Zymkey context.
 * @param battV
 *        (output) The current battery voltage value
 * @return 0 if successful, less than 0 for failure.
 */
int zkGetBatteryVoltage(zkCTX ctx,
                        float* batt_voltage);

/**@}*/

/** @name Battery Voltage Monitor
 */
/**@{*/


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
 * @return 0 for success, less than 0 for failure.
 */
int zkSetBatteryVoltageAction(zkCTX ctx, int action);

/**
 * @brief Sets the battery voltage threshold. (Supported Devices: HSM6, Secure Compute Module)
 * @details This function sets the threshold at which if the
 *          battery voltage falls bellow, the action set by
 *          zkSetBatteryVoltageAction will be carried out. The
 *          recommended threshold is 2.3V. If this
 *          function isn't called 2.3V is assumed by default. Threshold
 *          must be below 2.5V.
 * @param threshold
 *        (input) The threshold in Volts.
 * @return 0 for success, less than 0 for failure.
 */
int zkSetBatteryVoltageThreshold(zkCTX ctx, float threshold);

/**@}*/

/** @name CPU Temperature Monitor
 */
/**@{*/
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
 * @return 0 for success, less than 0 for failure.
 */
int zkSetCPUTempAction(zkCTX ctx, int action);


/**
 * @brief Sets the HSM CPU low temperature threshold. (Supported Devices: HSM6, Secure Compute Module)
 * @details This function sets the threshold at which if the
 *          on-board HSM CPU's tempreature falls below, the
 *          action set by zkSetCPUTempAction will be carried out.
 *          WARNING: You can lock yourself out in dev mode if you set
 *          a threshold above the CPU's ambient temperature.
 *          The recommended setting is no more than 20C.
 *          If no threshold is set, -10 degrees celsius is set as
 *          default.
 * @param threshold
 *        (input) The threshold in celsius.
 * @return 0 for success, less than 0 for failure.
 */
int zkSetCPULowTempThreshold(zkCTX ctx, float threshold);

/**
 * @brief Sets the HSM CPU high temperature threshold. (Supported Devices: HSM6, Secure Compute Module)
 * @details This function sets the threshold at which if the
 *          on-board HSM CPU's tempreature rises above, the
 *          action set by zkSetCPUTempAction will be carried out.
 *          WARNING: You can lock yourself out in dev mode if you set
 *          a threshold below the CPU's ambient temperature.
 *          The recommended setting is no less than 40C.
 *          If no threshold is set, 65 degrees celsius is set as
 *          default.
 * @param threshold
 *        (input) The threshold in celsius.
 * @return 0 for success, less than 0 for failure.
 */
int zkSetCPUHighTempThreshold(zkCTX ctx, float threshold);

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
int zkSetSupervisedBootPolicy(zkCTX ctx, int policy_id);

/**
 * @brief Update file manifest for Supervised boot to check. (Supported Devices: Secure Compute Module)
 * @details This function adds or updates files to be checked by Supervised boot.
 * @param file_path
 *        (input) The file to be signed and checked by Supervised boot.
 * @param slot
 *        (input) The slot to sign the file with.
 * @return 0 for success, less than 0 for failure.
 */
int zkAddOrUpdateSupervisedBootFile(zkCTX ctx, const char* file_path, int slot);

/**
 * @brief Remove a file from file manifest for Supervised boot to check. (Supported Devices: Secure Compute Module)
 * @details This function removes a file to be checked by Supervised boot.
 * @param file_path
 *        (input) The file to be removed from the Supervised boot manifest.
 * @return 0 for success, less than 0 for failure.
 */
int zkRemoveSupervisedBootFile(zkCTX ctx, const char* file_path);

/**
 * @brief Get file manifest for Supervised boot to check. (Supported Devices: Secure Compute Module)
 * @details This function gets the list of files to be checked by Supervised boot.
 * @param manifest
 *        (output) The file manifest that is checked by Supervised boot.
 * @return 0 for success, less than 0 for failure.
 */
int zkGetSupervisedBootFileManifest(zkCTX ctx, char** manifest);
/**@}*/

#ifdef __cplusplus
}
#endif // __cplusplus

#endif // __ZK_APP_UTILS_H

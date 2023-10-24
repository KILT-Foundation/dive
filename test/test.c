#include <stdio.h>
#include <zymkey/zk_app_utils.h>

const int MAX_NUM_KEYS = 32;

int list_keys(zkCTX ctx) {
    int max_num_keys = MAX_NUM_KEYS;
    int* alloc_key_list = NULL;
    int alloc_key_list_sz = 0;
    if (zkGetAllocSlotsList(ctx, false, &max_num_keys, &alloc_key_list, &alloc_key_list_sz) < 0) {
        printf("zk_get_alloc_slots_list failed\n");
        return -1;
    }
    printf("alloc_key_list_sz: %d\n", alloc_key_list_sz);
    for (int i = 0; i < alloc_key_list_sz; i++) {
        printf("alloc_key_list[%d]: %d\n", i, alloc_key_list[i]);
    }
    free(alloc_key_list);
}

int export_pubkey(zkCTX ctx, int slot) {
    uint8_t* pk = NULL;
    int pk_sz = 0;
    if (zkExportPubKey(ctx, (uint8_t**)&pk, &pk_sz, slot, false) < 0) {
        printf("zk_export_pubkey failed\n");
        return -1;
    }
    printf("pubkey:\n");
    for (int i = 0; i < pk_sz; i++) {
        printf("%02x", pk[i]);
    }
    printf("\n");
    free(pk);
}

int main(int argc, char** argv) {
    zkCTX ctx;
    if (zkOpen(&ctx) < 0) {
        printf("zk_open failed\n");
        return -1;
    }
    
    list_keys(ctx);
    export_pubkey(ctx, 0);

    if( zkClose(ctx) < 0) {
        printf("zk_close failed\n");
        return -1;
    }
}

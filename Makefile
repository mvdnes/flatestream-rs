ifndef OUT_DIR
$(error cargo should be used for building this library)
endif

$(OUT_DIR)/libminiz.a: csrc/miniz.c
	$(CC) -c csrc/miniz.c -fPIC -o $(OUT_DIR)/miniz.o
	$(AR) r $(OUT_DIR)/libminiz.a $(OUT_DIR)/miniz.o

# How much ZIM grows after unpacking:
# `du -s out ~/t/zim/wikipedia_ab_all_maxi_2022-05.zim`
growth_coefficient = 58960 / 19289

# Wikipedia ZIM size:
# https://dumps.wikimedia.org/other/kiwix/zim/wikipedia/wikipedia_en_all_maxi_2022-05.zim
zim_size = 95199730590

block_price = 4
blocks_freq = 1/5

unpacked_size = zim_size * growth_coefficient
print(f"unpacked size = {unpacked_size / 1024**3} GB")

unpacked_size_blocks = unpacked_size // 4096

price_per_second = unpacked_size_blocks * block_price * blocks_freq

time = 10 * 24 * 3600  # 10 days

print(f"cost in BZZ = {price_per_second * time / 10**16}")

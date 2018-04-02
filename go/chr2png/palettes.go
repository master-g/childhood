package main

import "encoding/hex"

const (
	palette3DSVC                        = "73737321188c0000ad42009c8c0073ad0010a500007b080042290000420000520000391018395a000000000000000000bdbdbd0073ef2139ef8400f7bd00bde7005ade2900ce4a088c730000940000ad0000943900848c101010000000000000ffffff39bdff5a94ffa58cfff77bffff73b5ff7363ff9c39f7bd3984d6104ade4a5aff9c00efde393939000000000000ffffffade7ffc6d6ffd6ceffffc6ffffc6deffbdb5ffdeadffe7a5e7ffa5adf7bdb5ffce9cfff78c8c8c000000000000"
	paletteASQRealityA                  = "60606000217b00009c31008b59006f6f00316400004f11002f190027290000440000393700394f0000000c0c0c0c0c0caeaeae1056ce1b2cff6020eca900bfca1654ca1a089e3a04675100436100007c000071530071870c0c0c0c0c0c0c0c0cffffff449efe5c6cff9966ffd760ffff6295ff6453f49430c2ac0090c41452d22820c69218bad24c4c4c0c0c0c0c0c0cffffffa3ccffa4b4ffc1b6ffe0b7ffffc0c5ffbcabffd09ffce090e2ea98caf2a0a0eae2a0e2fab6b6b60c0c0c0c0c0c"
	paletteASQRealityB                  = "6c6c6c0020940000a83c00987000707c003c700000681000461a003c2c00005000003c4c003a66000000101010101010bababa2a58d63c32ff8020f0c000c0d01474d21a24a63c1e7e520058640000880000746800729e101010101010101010ffffff5ea0ff8c82ffc470ffff5cffff68bcff727cfc964ad9ad0098c62e4ed44e3ac89a2ebedc585858101010101010ffffffc6d8ffd4cafff0c4ffffbcffffc4f0ffcad4ffd8beffe6a6eaecb2c6f4c6baeceab6e6ffc2c2c2101010101010"
	paletteBMFFinal2                    = "52525200008008008a2c007e4a004e5000064400002608000a2000002e0000320000260a001c48000000000000000000a4a4a40038ce3416ec5e04dc8c00b09a004c9018007036004c54000e6c00007400006c2c005e84000000000000000000ffffff4c9cff7c78ffa664ffda5afff054c0f06a56d68610baa40076c00046cc1a2ec86634c2be3a3a3a000000000000ffffffb6daffc8caffdac2fff0befffcbceefac2c0f2cca2e6da92cce68eb8eea2aeeabeaee8e2b0b0b0000000000000"
	paletteBMFFinal3                    = "6868680012991a08aa51029a7e00698e001c7e03015118001f3700014e00005a0000501c004061000000000000000000b9b9b90c5cd75035f08919e0bb0cb3ce0c61c02b0e954d01616f001f8b0001980c00934b00819b000000000000000000ffffff63b4ff9b91ffd377ffef6afff968c0f97d6ced9b2dbdbd167cda1c4be84735e5913fd9dd606060000000000000fffffface7ffd5cdffedbafff8b0fffeb0ecfdbdb5f9d28ee8eb7cbbf38299f7a28af5d092f4f1bebebe000000000000"
	paletteCommodore64                  = "8b8b8b4f449d4f449d4f449d4f449d6363636d530b6d530b6d530b6d530b6363634f449d4f449d000000000000000000cccbcc4f449d8a7fcda256a5a256a5a256a5a24d42a3683a5cad5f5cad5f5cad5f5cad5f5cad5f000000000000000000ffffff6ac2c8afafafafafafcd7f76cd7f76cd7f76cd7f769ce49d9ce49d9ce49d9ce49d6ac2c8636363000000000000ffffffcccccccccccccccccccccccccbd689cbd689cbd689cbd689ccccccccccccccccccccccccafafaf000000000000"
	paletteCompositeDirectFBX           = "65656500127d18008e36008256005d5a00184f05003819001d3100003d00004100003b17002e55000000000000000000afafaf194ec8472fe36b1fd7931bae9e1a5e9932007b4b005b6700267a00008200007a3e006e8a000000000000000000ffffff64a9ff8e89ffb676ffe06fffef6cc4f0806ad8982cb9b40a83cb0c5bd63f4ad17e4dc7cb4c4c4c000000000000ffffffc7e5ffd9d9ffe9d1fff9ceffffccf1ffd4cbf8dfb1edeaa4d6f4a4c5f8b8bef6d3bff1f1b9b9b9000000000000"
	paletteConsumer                     = "666666001e9a0e09a844009371006089011d861300692900393e00044c00004f0000472b00356c000000000000000000adadad0050f13b34ff8022e8bb1ea5db294ed74000b15e007379002d8b00008f08008460006db5000000000000000000ffffff4ba0ff8a84ffd172ffff6df7ff799eff9047ffae0ac4ca007ddc1341e15721d5b025beff4f4f4f000000000000ffffffb6d8ffd0cdffedc6ffffc4fcffc8d8ffd2b4ffde9ce7e994caf19fb2f3bba5eedfa6e5ffb8b8b8000000000000"
	paletteCRTNostalgiaBeta01           = "656565001f8e14049e32008e6000676700145d1000432600313400074600254600193c1e002861000000000000000000afafaf004adf2f28ff5026ff8920f4b0226ba53c008c50006e7000388000008600008a47006cad101010000000000000ffffff62aeff789bffaa7fffc96effee6deef09170f0b12fd9d000a0e60054e73f4ce58649ceff494949000000000000ffffffc2deffd4d8ffe4d8ffeed2fff8d3f8ffdfd3ffe9c0fdf59aebffa5c4ffb3bcfdd2beeaffb7b7b7000000000000"
	paletteFBXCompositeDCFinal          = "6b6b6b001e871f0b963b0c87590d615e0a1f5511003d2700233e00004b00004e0000461f00395c000000000000000000b2b2b21a53d14835ee7123ec9a1eb7a51e629d3705825400607100298400038b00008240017690000000000000000000ffffff63adfd908afeb977fce771fef76fc9f5836add9c29bdb80784d1075bdc3b48d77d4bcdcd555555000000000000ffffffc4e3fed7d5fee6cdfef9cafefec9f0fed1c7f7dcace8e89cd1f29dbff4b1b7f5cdb8f1edbebebe000000000000"
	paletteFBXCompositeDCFinalSaturated = "6b6b6b001da01f0cb2410ea0650f706c0b1f630e00452900234600005600005b0000501e003f69000000000000000000b2b2b20d55f04a2dfe791afeac1bcfbb1767b331008c55006079001d930000990000903e007c9f000000000000000000ffffff53acfd8e82fdbf6afef661fefe60d3fe7a5aeb990ac2be007cdb0049e12231e17334d4d8555555000000000000ffffffbbe3fed3d0fee9c7fefec3fefec3f2fecdbffad99eebe88bcdf48cb7f3a5aef4c8aef4f0bebebe000000000000"
	paletteFBXNESUnsaturated            = "6b6b6b001e871f0b963b0c87590d615e0528551100461b003032000a4800004e0000462400395c000000000000000000b2b2b21a53d14835ee7123ec9a1eb7a51e629d3705874b00676900298400038b00008240017690000000000000000000ffffff63adfd908afeb977fce771fef76fc9f5836add9c29bdb80784d1075bdc3b48d77d4bcdcd555555000000000000ffffffc4e3fed7d5fee6cdfef9cafefec9f0fed1c7f7dcace8e89cd1f29dbff4b1b7f5cdb8f1edbebebe000000000000"
	paletteFBXYUVCustomized             = "6b6b6b001e871f0b963b00915c007e70003c6c0800561d002e35000c4800005200004d2800315c000000000000000000b2b2b21a53d14835ee7123eca01accb71e7bb53120994e006b6d003887000d9300008c3c00789b000000000000000000ffffff63adfd908afeb977fce771feff6eccff8170ea9e22bdb80788d8005ce43046de8148cdde555555000000000000ffffffc0dfffd3d2ffe8c8fffac2ffffc9f0fed1c7f7d8a5e8e89cd1f29dbff4b1b7f5cdb8f1edbebebe000000000000"
	paletteFCEU13DefaultNitsuja         = "6060600000781400802c006e4a004e6c00185a0302511800342400003400003200003420002c78000000020202020202c4c4c40058de301ffc7f14e0a800b0c0065cc02b0ea640106f6100308000007c00007c3c006e84141414040404040404f0f0f04caaff6f73f5b070ffda5afff060c0f8836dd09030d4c03066d00026dd1a2ec86634c2be545454060606060606ffffffb6daffc8caffdac2fff0befffcbceeffd0b4ffda90ecec92dcf69eb8ffa2aeeabe9eefefbebebe080808080808"
	paletteFCEU15NitsujaNew             = "6060600000701400802c006e4a004e6c00185a0302511800342400003400003200003420002c78000000020202020202c4c4c40058de301ffc7f14e0a800b0c0065cc02b0ea640106f6100308000007c00007c3c006e84141414040404040404f0f0f04caaff6f73f5b070ffda5afff060c0f8836dd09030d4c03066d00026dd1a2ec86634c2be545454060606060606ffffffb6daffc8caffdac2fff0befffcbceeffd0b4ffda90ecec92dcf69eb8ffa2aeeabe9eefefbebebe080808080808"
	paletteFCEUX                        = "74747424188c0000a844009c8c0074a80010a400007c0800402c00004400005000003c14183c5c000000000000000000bcbcbc0070ec2038ec8000f0bc00bce40058d82800c84c0c88700000940000a800009038008088000000000000000000fcfcfc3cbcfc5c94fccc88fcf478fcfc74b4fc7460fc9838f0bc3c80d0104cdc4858f89800e8d8787878000000000000fcfcfca8e4fcc4d4fcd4c8fcfcc4fcfcc4d8fcbcb0fcd8a8fce4a0e0fca0a8f0bcb0fccc9cfcf0c4c4c4000000000000"
	paletteFX3NESPAL                    = "7880840000fc0000c44028c494008cac0028ac10008c1800503000007800006800005800004058000000000000000008bcc0c40078fc0088fc6848fcdc00d4e40060fc3800e46018ac800000b80000a80000a8480088942c2c2c000000000000fcf8fc38c0fc6888fc9c78fcfc78fcfc589cfc7858fca048fcb800bcf81858d85858f89c00e8e4606060000000000000fcf8fca4e8fcbcb8fcdcb8fcfcb8fcf4c0e0f4d0b4fce0b4fcd884dcf878b8f878b0f0d800f8fcc8c0c0000000000000"
	paletteGrayscale                    = "525252171717171717171717171717171717171717171717171717171717171717171717171717000000000000000000a0a0a0444444444444444444444444444444444444444444444444444444444444444444444444000000000000000000ffffff9797979797979797979797979797979797979797979797979797979797979797979797973c3c3c000000000000ffffffd3d3d3d3d3d3d3d3d3d3d3d3d3d3d3d3d3d3d3d3d3d3d3d3d3d3d3d3d3d3d3d3d3d3d3d3a9a9a9000000000000"
	paletteHybrid                       = "6c6c6d1217860e009c3e008f7100698100157a04005a11002f2e00004100004900003b170c3659000000000000000000b6b5b60b5edb3331ea760de6a90bb6c20c59b72c00a24a06716b0011860000940000843a00768a000000000000000000fdfdfd4eb1fd758cfdc07efdeb72fdf76ebbf67960ea9730d5b82280cc0c51d84350e48b25d7d3626263000000000000fdfdfdb5e2fdccd3fddbc9fdfac6fdfdc5e3fdc5bbf9d8abf3e4a0daf59fb4f1b7b4f7ccabf5efbfbfc0000000000000"
	paletteNESCAP                       = "6463650015801d009038008256005d5a001a4f0900381b001e3100003d00004100003a1b002f55000000000000000000afadaf164bca472ae76b1bdb9617b09f185b9630017b48005a6600237800017f0000783d006c8c000000000000000000ffffff60a6ff8f84ffb473ffe26cfff268c3ef7e61d89527bab30781c80757d43d47cf7e4bc5cd4c4b4d000000000000ffffffc2e0ffd5d2ffe3cbfff7c8fffec6eefecec6f6d7aee9e49fd3ed9dc0f2b2b9f1ccbaededbab9bb000000000000"
	paletteNESCLASSIC                   = "606060000088200c983814785414605c00105410003c240820340c0c400c184418003c20003058000000000000000000a8a8a80c4cc44c24e06814d09014ac9c1c489034047450045c6814187c101480081074481c6490000000000000000000fcfcfc6498fc887cfcb068fcdc6cf4e870ace48858cc9c20a8b00074c0005ccc5034c09050c0cc404040000000000000fcfcfcbcd4fcccccfcd8c4fcecc0fcf8c4e8f8ccc4e4cca8d8dc9cc8e4a0c0e4b8b4ecc8b8e4ecbababa000000000000"
	paletteNESCLASSICBeta               = "606060000088200c983814785414605c00105410003c240820340c0c400c184418003c20003058000000000000000000a8a8a80c4cc44c24e06814d09014ac9c1c489034047450045c6814187c101480081074481c6490000000000000000000fcfcfc6498fc887cfcb068fcdc6cf4e870ace48858cc9c20a8b00074c0005ccc5034c09050c0cc404040000000000000fcfcfcbcd4fcccccfcd8c4fcecc0fcf8c4e8f8ccc4e4cca8d8dc9cc8e4a0c0e4b8b4ecc8b8e4ecbababa000000000000"
	paletteNESClassicFBX                = "6060600000831f069e380f7c560c625b0010530c003a230820350b0c410b194516023e1e023154000000000000000000a9a9a9104bbf4a1ee4690ad28e12b29e0f4c8f32047351065c6a12187d101481091175471d668f000000000000000000fbfbfb6699f88978feb262ffde63ffeb69b3e38758c89f22a7b10373c2035dd04f36c58d50c5cc404040000000000000fbfbfbbfd4facdcbfed9c2ffecbefffac2ebf7cac3e3cda7d9de9cc8e69ec0e6b8b5edc7b9e6eab8b8b8000000000000"
	paletteNESClassicFBXfs              = "60615f0000831d019534087551055e56000f4c0700372308203a0b0f4b0e194c1602421e023154000000000000000000a9aaa8104bbf4712d86300ca8800a9930b468a2d046f52065c71141b8d12199509178448206b8e000000000000000000fbfbfb6699f88974f9ab58f8d557efde5fa9dc7f59c7a224a7be0375d70360e34f3cd68d56c9cc414240000000000000fbfbfbbed4fac9c7f9d7befae8b8f9f5bae5f3cac2dfcda7d9e09cc9eb9ec0edb8b5f4c7b9eae9ababab000000000000"
	paletteNESRemixU                    = "6e6e6e2a159c2a0eab5c1c958e23729c1c3195310e72400747550739630e3b650d315c40314e72000000000000000000a2a2a23955d53131dc7931ceb931a3c03155c04600a35c00797900478e0040950e408e5c3179956e6e6e000000000000dcdcdc5c8edc4772dc8e63dcce55dcdc5595dc6a31dc8e07c7b21c8ac70763ce395cc78055b2b9a2a2a2000000000000dcdcdc87b9dc8ea3dcab9cdcc79cdcdc9cc7dcab9cdcc087dcce79b2d5728ed58e8edcb28edccedcdcdc000000000000"
	paletteNintendulatorNTSC            = "656565002b9b110ec03f00bc66008f7b0045790100601c00363800084f00005a00005702004555000000000000000000aeaeae0761f53e3bff7c1dffaf0ee5cb1383c82a15a74d006f7200329100009f00009b2a008498000000000000000000ffffff56b1ff8e8bffcc6cffff5dffff62d4ff7964f89d06c0c30081e2004df11630ec7a34d5ea4e4e4e000000000000ffffffbadfffd1d0ffebc3ffffbdffffbfeeffc8c0fcd799e5e784ccf387b6f9a0aaf8c9aceef7b7b7b7000000000000"
	paletteNostalgiaFBX                 = "65656500127d18008e36008256005d5a00184f05003819001d3100003d00004100003b17002e55000000000000000000afafaf194ec8472fe36b1fd7931bae9e1a5e9732007b4b005b6700267a00008200007a3e006e8a000000000000000000ffffff64a9ff8e89ffb676ffe06fffef6cc4f0806ad8982cb9b40a83cb0c5bd63f4ad17e4dc7cb4c4c4c000000000000ffffffc7e5ffd9d9ffe9d1fff9ceffffccf1ffd4cbf8dfb1edeaa4d6f4a4c5f8b8bef6d3bff1f1b9b9b9000000000000"
	paletteNTSCU                        = "6463660015811e009038008356005c5a0019500a00391d001f3200003e00004100003a1a003057000000000000000000afaeb0174bcb472be76c1bdc9717b2a0185b9830007b49005b6600257800007f0000793e006c8c000000000000000000ffffff62a8ff9185ffb474ffe26cfff369c3f17f64d99528bbb50781c90958d53f48d07e4cc7cd4d4c4e000000000000ffffffc3e1ffd6d3ffe4cbfff8c8ffffc7eeffcfc7f6d8afe9e5a0d4ed9ec1f2b3baf2cebbeeedbbbabb000000000000"
	paletteOriginalHardwareFBX          = "6a6d6a00127d1800823b007d56005d5a00184f0d00381e00203100003d00004000003b1e002e55000000000000000000b9bcb9194ec8472fe3751fd7931ead9e245e9638007b50005b6700267a00007f00007842006e8a000000000000000000ffffff69aeff9798ffb687ffe278fff279c7f58f6fdda932bcb70d88d01560db494fd68750cace515451000000000000ffffffcceaffdee2ffeedafffad7fdfdd7f6fddcd0fae8b6f2f1a9dbfba9caffbdc3fbd8c4f6f6bec1be000000000000"
	palettePVMtoDigitalFBXbeta01        = "787864001291230a963c008c6000676e00195f19004926002e3600004800004b0000461e003753000000000000000000c3c3a5194ec8472fe37528e1aa1baebe2850b43c00965a006e7200418700008c00008933007f91000000000000000000ffffff64a9ff9d9bffc78fffef82fff880c8f88c5cf2b22cdccd0a95e40c69ea414ae7824ddade555541000000000000ffffffd2eaffe9e7fff2d8fff8d2fffcd7edffe2beffee9df3f27dd3fa91b6ffa6a9ffc5c3f8f8bebeaa000000000000"
	palettePVMtoDigitalFBXbeta02        = "69696400127d18008e36008256005d5a000f4f0a00381d00263000003d00004100003b1400324e000000000000000000b9b9b41449c3422ade7023dca516a9b41e46ab3c00965500696d003c820000870000842e007a8c000000000000000000ffffff5fa4fa9896fac28afaea7dfaf387b4f08b5bedad27d7c80590df0764e53c45e27d48d5d94b4b46000000000000ffffffd2eaffe9e7fff2d8fff8d2fffcd7edffe2befbea9df3f28cd3fa91b6ffa6a9ffc5c3f8f8b4b4af000000000000"
	palettePVMtoDigitalFBXbeta03        = "69696400127d28007d3a007d5d00545f00145614003821002a3000143a00004100003b1e003050000000000000000000b9b9b41449c34d2cda7023dca516a9aa2b51a13e00895300656d002c7900008100007d4200788a000000000000000000ffffff69a8ff9c97ffc28afaea7dfaf387b4f0936ae3b427d7c80590df0764e53c45e27d48d5d94b4b46000000000000ffffffd2eaffe9e7fff2d8fff8d2fffcd7edffe2befbea9df3f28cd3fa91b6ffa6a9ffc5c3f8f8bebeb9000000000000"
	palettePVMtoDigitalFBXbeta04        = "69696400177428007d4100705600575e0013531a003b24002a3000143a00004100003b1e003050000000000000000000b9b9b41453b94d2cda7a1ec898189c9d2344a03e008d5500656d002c7900008100007d4200788a000000000000000000ffffff69a8ff9a96ffc28afaea7dfaf387b4f19367e6b327d7c80590df0764e53c45e27d48d5d94b4b46000000000000ffffffd2eaffe2e2fff2d8fff8d2fffcd7edffe2befbea9df3f28cd3fa91b6ffa6a9ffc5c3f8f8bebeb9000000000000"
	paletteRaw                          = "000000110000220000330000440000550000660000770000880000990000aa0000bb0000cc0000ff0000ee0000ff0000005500115500225500335500445500555500665500775500885500995500aa5500bb5500cc5500dd5500ee5500ff550000aa0011aa0022aa0033aa0044aa0055aa0066aa0077aa0088aa0099aa00aaaa00bbaa00ccaa00ddaa00eeaa00ffaa0000ff0011ff0022ff0033ff0044ff0055ff0066ff0077ff0088ff0099ff00aaff00bbff00ccff00ddff00eeff00ffff00"
	paletteRGB                          = "6d6d6d0024920000db6d49db92006db6006db624009249006d4900244900006d24009200004949000000000000000000b6b6b6006ddb0049ff9200ffb600ffff0092ff0000db6d00926d0024920000920000b66d009292242424000000000000ffffff6db6ff9292ffdb6dffff00ffff6dffff9200ffb600dbdb006ddb0000ff0049ffdb00ffff494949000000000000ffffffb6dbffdbb6ffffb6ffff92ffffb6b6ffdb92ffff49ffff6db6ff4992ff6d49ffdb92dbff929292000000000000"
	paletteRockman9                     = "7070700000a8201888400098880070a80010a00000780800402800004000005000003810183858000000000000000000b8b8b80070e82038e88000f0b800b8e00058d82800c8480888700000900000a800009038008088000000000000000000f8f8f838b8f85890f8a088f8f078f8f870b0f87060f89838f0b83880d01048d84858f89800e8d8505050000000000000f8f8f8a8e0f8c0d0f8d0c8f8f8c0f8f8c0d8f8b8b0f8d8a8f8e0a0e0f8a0a8f0b8b0f8c898f8f0989898000000000000"
	paletteRockman921to2C               = "7070700000a8201888400098880070a80010a00000780800402800004000005000003810183858000000000000000000b8b8b80070e82038e88000f0b800b8e00058d82800c8480888700000900000a800009038008088000000000000000000f8f8f838b8f85890f8a088f8f078f8f870b0f87060f89838f0b83880d01048d84858f89838b8f8505050000000000000f8f8f8a8e0f8c0d0f8d0c8f8f8c0f8f8c0d8f8b8b0f8d8a8f8e0a0e0f8a0a8f0b8b0f8c898f8f0989898000000000000"
	paletteSony                         = "58585800238c00139b2d05855d00527a00177a08005f1800352a00093900003f00003c2200325d000000000000000000a1a1a10053ee153cfe6028e4a91d98d41e41d22c00aa44006c5e002d7300007d060078520069a9000000000000000000ffffff1fa5fe5e89feb572fefe65f6fe6790fe773cfe9308c4b20079ca103ad54a11d1a406bffe424242000000000000ffffffa0d9febdccfee1c2fefebcfbfebdd0fec5a9fed18ee9de86c7e992a8eeb095ecd991e4feacacac000000000000"
	paletteUnsaturatedFinal             = "676767001f8e23069e40008e60006767001c5b1000432500313400074800004f00004622003a61000000000000000000b3b3b3205adf5138fb7a27eea520c2b0226bad37028d56006e70002e8a00069200008a47037b9b101010000000000000ffffff62aeff918bffbc78ffe96efffc6ccdfa8267e29b26c0b90184d20058de3846d97d49ced2494949000000000000ffffffc1e3ffd5d4ffe7ccfffbc9ffffc7f0ffd0c5f8daaaebe69ad1f19abef7afb6f4cdb7f0efb2b2b2000000000000"
	paletteUnsaturatedV4                = "6b6b6b001e871f0b963b0c87590d615e0528551100461b003032000a4800004e0000461900395c000000000000000000b2b2b21a53d14835ee7123ec9a1eb7a51e62a52d19874b00676900298400038b00008240017690000000000000000000ffffff63adfd908afeb977fce771fef76fc9f5836add9c29bdb80784d1075bdc3b48d77d4bcdcd555555000000000000ffffffc4e3fed7d5fee6cdfef9cafefec9f0fed1c7f7dcace8e89cd1f29dbff4b1b7f5cdb8f1edbebebe000000000000"
	paletteUnsaturatedV5                = "6b6b6b001e871f0b963b0c87590d615e0528551100461b003032000a4800004e0000461900395a000000000000000000b2b2b21a53d14835ee7123ec9a1eb7a51e62a52d19874b00676900298400038b00008240007096000000000000000000ffffff63adfd908afeb977fce771fef76fc9f5836add9c29bdb80784d1075bdc3b48d77d48c6d8555555000000000000ffffffc4e3fed7d5fee6cdfef9cafefec9f0fed1c7f7dcace8e89cd1f29dbff4b1b7f5cdb7ebf2bebebe000000000000"
	paletteUnsaturatedV6                = "6b6b6b001e871f0b963b0c87590d615e0528551100461b003032000a4800004e00004619003a58000000000000000000b2b2b21a53d14835ee7123ec9a1eb7a51e62a52d19874b00676900298400038b00008240007891000000000000000000ffffff63adfd908afeb977fce771fef76fc9f5836add9c29bdb80784d1075bdc3b48d77d48ccce555555000000000000ffffffc4e3fed7d5fee6cdfef9cafefec9f0fed1c7f7dcace8e89cd1f29dbff4b1b7f5cdb7f0eebebebe000000000000"
	paletteUnsaturatedV7                = "6f6f6f00108a1300973400885900615a00195500003d1c002d2f00034600004b0000421800315d000000000000000000b9b9b91952d74a31f3711fe7a01abda91a63a13400824f006a6b00278300008b00008341007695000000000000000000ffffff62acff918bffbd79ffe96efffb6ccefa8169df9b24bfb80183d00157dd3844d87c47cdcf4e4e4e000000000000ffffffc4e4ffd7d7ffe9cefffccbffffc9f2ffd2c9f9dcacebe89cd2f39bbff8b2b7f6cebaf1f1b8b8b8000000000000"
	paletteWiiVC                        = "49494900006a09006329005942004a4900004200002911001827000030100030000029100120430000000000000000007471740030843101ac4b019464007b6b00396b21015a2f0042490018590110590101593201495a101010000000000000adadad4a71b66458d58450e6a451adad4984b5624a9471327b722a5a8601388e31318e5a398e8d383838000000000000b6b6b68c9db58d8eae9c8ebca687bcad8d9dae968c9c8f7c9c9e7294a67c84a77b7c9d8473968ddedede000000000000"
	paletteWiiVCbrighter                = "6666660000950d008b39007d5c00686600005c0000391800223700004316004300003916012d5e000000000000000000a39ea30043b94501f16901cf8c00ac960050962e017e42005c6600227d01167d01017d4601667e101010000000000000f2f2f2689eff8c7bffb970ffe671f2f266b9fe8968cf9e46aca03b7ebc014ec74545c77e50c7c6383838000000000000ffffffc4dcfec6c7f4dbc7ffe9bdfff2c6dcf4d2c4dbc8aedbdda0cfe9aeb9eaacaedcb9a1d2c6dedede000000000000"
	palettexvzgjw                       = "63636300008f0c0086370078590064630000590000371700203400004115004100003715012b5a0000000000000000009d999d0041b34201e96501c88700a691004d912c017a3f0059630020780115780101784301637a151515000000000000eaeaea6499f68777ffb36cffde6deaea63b3f58464c89943a69a387ab5014bc04242c07a4dc0bf4b4b4b000000000000f6f6f6bdd4f5bfc0ecd3c0ffe1b7ffeabfd4eccbbdd3c1a8d3d69ac8e1a8b3e2a6a8d4b39bcbbfffffff000000000000"
	paletteYUV                          = "666666002a881412a73b00a45c007e6e00406c0700561d003335000c4800005200004f0800404d000000000000000000adadad155fd94240ff7527fea01accb71e7bb53120994e006b6d003887000d9300008f32007c8d000000000000000000ffffff64b0ff9290ffc676fff26affff6eccff8170ea9e22bcbe0088d8005ce43045e08248cdde4f4f4f000000000000ffffffc0dfffd3d2ffe8c8fffac2ffffc4eaffccc5f7d8a5e4e594cfef96bdf4abb3f3ccb5ebf2b8b8b8000000000000"
	paletteYUVV3                        = "666666002a881412a73b00a45c007e6e00406c0700561d003335000c4800005200004c18003e5b000000000000000000adadad155fd94240ff7527fea01accb71e7bb53120994e006b6d003887000d9300008c47007aa0000000000000000000ffffff64b0ff9290ffc676fff26affff6eccff8170ea9e22bcbe0088d8005ce43045e08248cdde4f4f4f000000000000ffffffc0dfffd3d2ffe8c8fffac2ffffc4eaffccc5f7d8a5e4e594cfef96bdf4abb3f3ccb5ebf2b8b8b8000000000000"
	paletteYUVCorrected                 = "666666002a881412a73b00a45c007e6e00406c0700561d003335000c4800005200004d28003f69000000000000000000adadad155fd94240ff7527fea01accb71e7bb53120994e006b6d003887000d9300008c3c007c9f000000000000000000ffffff64b0ff9290ffc676fff26affff6eccff8170ea9e22bcbe0088d8005ce43045e08248cdde4f4f4f000000000000ffffffc0dfffd3d2ffe8c8fffac2ffffc4eaffccc5f7d8a5e4e594cfef96bdf4abb3f3ccb5ebf2b8b8b8000000000000"
)

var (
	paletteMap map[string]string
)

func init() {
	paletteMap = map[string]string{
		"3DSVC":                        palette3DSVC,
		"ASQRealityA":                  paletteASQRealityA,
		"ASQRealityB":                  paletteASQRealityB,
		"BMFFinal2":                    paletteBMFFinal2,
		"BMFFinal3":                    paletteBMFFinal3,
		"Commodore64":                  paletteCommodore64,
		"CompositeDirectFBX":           paletteCompositeDirectFBX,
		"Consumer":                     paletteConsumer,
		"CRTNostalgiaBeta01":           paletteCRTNostalgiaBeta01,
		"FBXCompositeDCFinal":          paletteFBXCompositeDCFinal,
		"FBXCompositeDCFinalSaturated": paletteFBXCompositeDCFinalSaturated,
		"FBXNESUnsaturated":            paletteFBXNESUnsaturated,
		"FBXYUVCustomized":             paletteFBXYUVCustomized,
		"FCEU13DefaultNitsuja":         paletteFCEU13DefaultNitsuja,
		"FCEU15NitsujaNew":             paletteFCEU15NitsujaNew,
		"FCEUX":                        paletteFCEUX,
		"FX3NESPAL":                    paletteFX3NESPAL,
		"Grayscale":                    paletteGrayscale,
		"Hybrid":                       paletteHybrid,
		"NESCAP":                       paletteNESCAP,
		"NESCLASSIC":                   paletteNESCLASSIC,
		"NESCLASSICBeta":               paletteNESCLASSICBeta,
		"NESClassicFBX":                paletteNESClassicFBX,
		"NESClassicFBXfs":              paletteNESClassicFBXfs,
		"NESRemixU":                    paletteNESRemixU,
		"NintendulatorNTSC":            paletteNintendulatorNTSC,
		"NostalgiaFBX":                 paletteNostalgiaFBX,
		"NTSCU":                        paletteNTSCU,
		"OriginalHardwareFBX":          paletteOriginalHardwareFBX,
		"PVMtoDigitalFBXbeta01":        palettePVMtoDigitalFBXbeta01,
		"PVMtoDigitalFBXbeta02":        palettePVMtoDigitalFBXbeta02,
		"PVMtoDigitalFBXbeta03":        palettePVMtoDigitalFBXbeta03,
		"PVMtoDigitalFBXbeta04":        palettePVMtoDigitalFBXbeta04,
		"Raw":              paletteRaw,
		"RGB":              paletteRGB,
		"Rockman9":         paletteRockman9,
		"Rockman921to2C":   paletteRockman921to2C,
		"Sony":             paletteSony,
		"UnsaturatedFinal": paletteUnsaturatedFinal,
		"UnsaturatedV4":    paletteUnsaturatedV4,
		"UnsaturatedV5":    paletteUnsaturatedV5,
		"UnsaturatedV6":    paletteUnsaturatedV6,
		"UnsaturatedV7":    paletteUnsaturatedV7,
		"WiiVC":            paletteWiiVC,
		"WiiVCbrighter":    paletteWiiVCbrighter,
		"xvzgjw":           palettexvzgjw,
		"YUV":              paletteYUV,
		"YUVV3":            paletteYUVV3,
		"YUVCorrected":     paletteYUVCorrected,
	}
}

func getPalette(name string) []byte {
	if raw, ok := paletteMap[name]; ok {
		p, err := hex.DecodeString(raw)
		if err != nil {
			return nil
		} else {
			return p
		}
	}

	return nil
}
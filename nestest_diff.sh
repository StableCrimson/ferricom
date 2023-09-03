cargo r roms/nestest.nes -c -d > logs/output.log
diff -y logs/output.log logs/nestest.log > diff.diff
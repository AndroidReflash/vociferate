# Vociferate

KMP-проект для винды, линукса и андроида. 
Если между компами единая локалка - сойдёт как говорилка
rust.sh чтобы собирать бинари если вдруг надо. 
на линуксе jar-файл для винды собирать через градл (раскомментировать строку с пометкой "винда") - ./gradlew packageWindowsFatJar
на линуксе jar-файл для линукса собирать через градл (раскомментировать строку с пометкой "линукс") - ./gradlew packageFatJar

Если ключи делать самостоятельно:
1) openssl genrsa -out key.pem 2048
2) openssl req -new -x509 -key key.pem -out cert.pem -days 36500 -subj "/CN=localhost"
3) openssl pkcs8 -topk8 -nocrypt -in key.pem -outform DER -out key.der
4) openssl x509 -in cert.pem -outform DER -out cert.der
5) в папке rust (рядом с src, Cargo.lock и Cargo.toml) создать папку assets
6) закинуть туда получившиеся cert.der и key.der
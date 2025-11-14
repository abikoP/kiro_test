# 概要
ブラウザ上でtelegraph.confファイルで管理しているurl setを編集できるWEB applycation

# 要件
Telegrachで監視するURLsetを管理するWEB applucation  
telegraph.conf.sumple 内の下記部分を編集する．  
```
  urls = [ "https://amazon.com","https:///"]
```
編集は認証されたユーザのみができる．



## 言語およびフレームワーク
rustのlocoを使って作成．

## 仕様
・url SETをブラウザ上で確認できるページと，管理画面をもつ．  
・管理画面ではログイン制御し，ログインしたユーザがURL setを編集できる．  
・ログイン時には，user_nameとpasswordを入力する  
・編集画面で編集し，保存すると，実際のconfファイルに設定を反映する．  
・confファイルの設置PATHは，./conf/telegraf.conf  
・現在設定中のURLを一覧表示できること  
・複数個のURLをまとめて追加/削除できること  
・URL追加の際は，URLのvalidationを行うこと

## ページ構成
・/conf : 今現在のtelegraph.confを表示する.  
・/admin/ 管理画面トップページ  
・/admin/edit/ 編集画面  
・/admin/list/ 一覧画面

## lint
Clippy を用いた整合性チェックを都度行うこと  

# 機能追加
現段階では未定だが，今後telegraphの管理ツールとして運用することが想定される．

# 参考
telegraph.confの仕様については下記ドキュメントを参照されたし  
https://docs.influxdata.com/telegraf/v1/configuration/


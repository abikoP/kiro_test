# 概要
管理画面に，docker restart telegraf を実行するボタンを設置する．

## 詳細
このシステムは，telegraf.confのURL listを編集するためのシステムである．  
telegrafは，influxDBとgrafanaとともに，dockerのcontainerとして稼働している．  
そのため，telegraf.confを編集した際には，telefrafのdocker containerを再起動する必要がある．  

## 仕様
- /admin のヘッダ箇所，URL編集とログアウトボタンの間に，Telegraf再起動の項目を設置
- 押下時，再起動してもいいかのアラートウィンドウを表示．
  - はいを選んだ場合はdocker-compose restart telegraf を実行．
    - コマンド実行時，エラーを確認した場合はフロントエンド(アラートウィンドウ)に表示．
    - 正常終了した場合はその旨表示させて終了．
  - いいえを選んだ場合は何もしない．
## 補足
URL setの更新(登録，削除)時にはコンテナの再起動はしない．

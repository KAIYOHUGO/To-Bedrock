# to bedrock

![icon](https://i.imgur.com/yprFoFr.png)

_a better bedrock translate tool_

it will make bedrock version translate be the same as java version translate

## How to use

1. download tobedrock.exe from releases
2. run it
3. follow step print in the screen
4. open betterbedrocktranslate.mcpack

## Build from src

```bash
# build file
go build .

# run it
./tobedrock

# or
tobedrock.exe
```

## Example
```bash

$ tobedrock.exe

input lang type you want to translate to:zh_TW      
input java version en_us lang file:example/en_us.json
input bedrock version en_us lang file:example/en_us.lang
input java version lang file you want to translate to:example/zh_tw.json
input bedrock version lang file to compose (can omit):example/zh_tw.lang
pack up : to bedrock/template/
pack up : to bedrock/template/manifest.json
pack up : to bedrock/template/pack_icon.png
pack up : to bedrock/template/texts
pack up : to bedrock/template/texts/zh_TW.lang
done
```

<div>java icons made by <a href="https://www.freepik.com" title="Freepik">Freepik</a> from <a href="https://www.flaticon.com/" title="Flaticon">www.flaticon.com</a></div>
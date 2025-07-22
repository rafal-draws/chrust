#!/bin/sh 
set -x 
cd /metadata 
rm *
gdown --fuzzy https://drive.google.com/file/d/1gsGht48Jc2h6zIRygEXqr67ceeDobpIh/view?usp=sharing
unzip models_and_metadata.zip 

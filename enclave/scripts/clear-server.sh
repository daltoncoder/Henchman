#!/bin/bash
BASEDIR="$( cd -- "$( dirname -- "${BASH_SOURCE[0]}" )/.." &> /dev/null && pwd )"
SCRIPTSPATH="$BASEDIR/scripts/"
GRAMINEPATH="$BASEDIR/gramine/"
SEALPATH="$GRAMINEPATH/seal/"
CERTPATH="$GRAMINEPATH/certificates/"

# DEFAULT VALUES
HTTPS_PUBLIC_KEY=${HTTPS_PUBLIC_KEY:-$CERTPATH/server_cert.pem}
HTTPS_PRIVATE_KEY=${HTTPS_PRIVATE_KEY:-$CERTPATH/server_key.pem}
SEAL_PATH=${SEAL_PATH:-$SEALPATH}
ENCLAVE_IDENTITY=${ENCLAVE_IDENTITY:-TEE-1}

stop_enclave() {
    printf 'stop enclave with identifier : "%s"\n' "$1" >&2
    ps aux | grep "$1" | grep -v grep | awk '{ print $2}' | xargs kill -9
}

die () {
    printf '%s\n' "$1" >&2
    exit
}

while :; do
    case $1 in
        -p|--port)
	    if [ "$2" ]; then
		    PORTID="port $2"
            stop_enclave "$PORTID"
		    shift
	    else
		    die 'ERROR: "--port" requires a non-empty option argument.'
	    fi
        ;;
        *) break
    esac
    shift
done

find . -name "*~" | xargs rm
rm -rf $GRAMINEPATH/certificates/*
rm -rf $SEALPATH/*.keyshare
rm -rf $SEALPATH/*.log
#rm -rf $GRAMINEPATH/bin/*

cd $GRAMINEPATH
make clean
cd $BASEDIR

FROM rustembedded/cross:x86_64-unknown-linux-gnu

COPY stage/centos-packages /
RUN yum install -y epel-release && yum repolist && yum update -y && yum install -y `cat /centos-packages`

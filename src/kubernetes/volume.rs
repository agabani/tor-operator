use k8s_openapi::api::core::v1::{
    AWSElasticBlockStoreVolumeSource, AzureDiskVolumeSource, AzureFileVolumeSource,
    CSIVolumeSource, CephFSVolumeSource, CinderVolumeSource, ConfigMapVolumeSource,
    DownwardAPIVolumeSource, EmptyDirVolumeSource, EphemeralVolumeSource, FCVolumeSource,
    FlexVolumeSource, FlockerVolumeSource, GCEPersistentDiskVolumeSource, GitRepoVolumeSource,
    GlusterfsVolumeSource, HostPathVolumeSource, ISCSIVolumeSource, NFSVolumeSource,
    PersistentVolumeClaimVolumeSource, PhotonPersistentDiskVolumeSource, PortworxVolumeSource,
    ProjectedVolumeSource, QuobyteVolumeSource, RBDVolumeSource, ScaleIOVolumeSource,
    SecretVolumeSource, StorageOSVolumeSource, VsphereVirtualDiskVolumeSource,
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Volume represents a named volume in a pod that may be accessed by any container in the pod.
#[derive(JsonSchema, Deserialize, Serialize, Clone, Debug, Default, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Volume {
    /// awsElasticBlockStore represents an AWS Disk resource that is attached to a kubelet's host machine and then exposed to the pod. More info: https://kubernetes.io/docs/concepts/storage/volumes#awselasticblockstore
    pub aws_elastic_block_store: Option<AWSElasticBlockStoreVolumeSource>,

    /// azureDisk represents an Azure Data Disk mount on the host and bind mount to the pod.
    pub azure_disk: Option<AzureDiskVolumeSource>,

    /// azureFile represents an Azure File Service mount on the host and bind mount to the pod.
    pub azure_file: Option<AzureFileVolumeSource>,

    /// cephFS represents a Ceph FS mount on the host that shares a pod's lifetime
    pub cephfs: Option<CephFSVolumeSource>,

    /// cinder represents a cinder volume attached and mounted on kubelets host machine. More info: https://examples.k8s.io/mysql-cinder-pd/README.md
    pub cinder: Option<CinderVolumeSource>,

    /// configMap represents a configMap that should populate this volume
    pub config_map: Option<ConfigMapVolumeSource>,

    /// csi (Container Storage Interface) represents ephemeral storage that is handled by certain external CSI drivers (Beta feature).
    pub csi: Option<CSIVolumeSource>,

    /// downwardAPI represents downward API about the pod that should populate this volume
    pub downward_api: Option<DownwardAPIVolumeSource>,

    /// emptyDir represents a temporary directory that shares a pod's lifetime. More info: https://kubernetes.io/docs/concepts/storage/volumes#emptydir
    pub empty_dir: Option<EmptyDirVolumeSource>,

    /// ephemeral represents a volume that is handled by a cluster storage driver. The volume's lifecycle is tied to the pod that defines it - it will be created before the pod starts, and deleted when the pod is removed.
    ///
    /// Use this if: a) the volume is only needed while the pod runs, b) features of normal volumes like restoring from snapshot or capacity
    ///    tracking are needed,
    /// c) the storage driver is specified through a storage class, and d) the storage driver supports dynamic volume provisioning through
    ///    a PersistentVolumeClaim (see EphemeralVolumeSource for more
    ///    information on the connection between this volume type
    ///    and PersistentVolumeClaim).
    ///
    /// Use PersistentVolumeClaim or one of the vendor-specific APIs for volumes that persist for longer than the lifecycle of an individual pod.
    ///
    /// Use CSI for light-weight local ephemeral volumes if the CSI driver is meant to be used that way - see the documentation of the driver for more information.
    ///
    /// A pod can use both types of ephemeral volumes and persistent volumes at the same time.
    pub ephemeral: Option<EphemeralVolumeSource>,

    /// fc represents a Fibre Channel resource that is attached to a kubelet's host machine and then exposed to the pod.
    pub fc: Option<FCVolumeSource>,

    /// flexVolume represents a generic volume resource that is provisioned/attached using an exec based plugin.
    #[allow(clippy::struct_field_names)]
    pub flex_volume: Option<FlexVolumeSource>,

    /// flocker represents a Flocker volume attached to a kubelet's host machine. This depends on the Flocker control service being running
    pub flocker: Option<FlockerVolumeSource>,

    /// gcePersistentDisk represents a GCE Disk resource that is attached to a kubelet's host machine and then exposed to the pod. More info: https://kubernetes.io/docs/concepts/storage/volumes#gcepersistentdisk
    pub gce_persistent_disk: Option<GCEPersistentDiskVolumeSource>,

    /// gitRepo represents a git repository at a particular revision. DEPRECATED: GitRepo is deprecated. To provision a container with a git repo, mount an EmptyDir into an InitContainer that clones the repo using git, then mount the EmptyDir into the Pod's container.
    pub git_repo: Option<GitRepoVolumeSource>,

    /// glusterfs represents a Glusterfs mount on the host that shares a pod's lifetime. More info: https://examples.k8s.io/volumes/glusterfs/README.md
    pub glusterfs: Option<GlusterfsVolumeSource>,

    /// hostPath represents a pre-existing file or directory on the host machine that is directly exposed to the container. This is generally used for system agents or other privileged things that are allowed to see the host machine. Most containers will NOT need this. More info: https://kubernetes.io/docs/concepts/storage/volumes#hostpath
    pub host_path: Option<HostPathVolumeSource>,

    /// iscsi represents an ISCSI Disk resource that is attached to a kubelet's host machine and then exposed to the pod. More info: https://examples.k8s.io/volumes/iscsi/README.md
    pub iscsi: Option<ISCSIVolumeSource>,

    /// nfs represents an NFS mount on the host that shares a pod's lifetime More info: https://kubernetes.io/docs/concepts/storage/volumes#nfs
    pub nfs: Option<NFSVolumeSource>,

    /// persistentVolumeClaimVolumeSource represents a reference to a PersistentVolumeClaim in the same namespace. More info: https://kubernetes.io/docs/concepts/storage/persistent-volumes#persistentvolumeclaims
    pub persistent_volume_claim: Option<PersistentVolumeClaimVolumeSource>,

    /// photonPersistentDisk represents a PhotonController persistent disk attached and mounted on kubelets host machine
    pub photon_persistent_disk: Option<PhotonPersistentDiskVolumeSource>,

    /// portworxVolume represents a portworx volume attached and mounted on kubelets host machine
    #[allow(clippy::struct_field_names)]
    pub portworx_volume: Option<PortworxVolumeSource>,

    /// projected items for all in one resources secrets, configmaps, and downward API
    pub projected: Option<ProjectedVolumeSource>,

    /// quobyte represents a Quobyte mount on the host that shares a pod's lifetime
    pub quobyte: Option<QuobyteVolumeSource>,

    /// rbd represents a Rados Block Device mount on the host that shares a pod's lifetime. More info: https://examples.k8s.io/volumes/rbd/README.md
    pub rbd: Option<RBDVolumeSource>,

    /// scaleIO represents a ScaleIO persistent volume attached and mounted on Kubernetes nodes.
    pub scale_io: Option<ScaleIOVolumeSource>,

    /// secret represents a secret that should populate this volume. More info: https://kubernetes.io/docs/concepts/storage/volumes#secret
    pub secret: Option<SecretVolumeSource>,

    /// storageOS represents a StorageOS volume attached and mounted on Kubernetes nodes.
    pub storageos: Option<StorageOSVolumeSource>,

    /// vsphereVolume represents a vSphere volume attached and mounted on kubelets host machine
    #[allow(clippy::struct_field_names)]
    pub vsphere_volume: Option<VsphereVirtualDiskVolumeSource>,
}

impl Volume {
    pub fn into_volume(self, name: String) -> k8s_openapi::api::core::v1::Volume {
        k8s_openapi::api::core::v1::Volume {
            aws_elastic_block_store: self.aws_elastic_block_store,
            azure_disk: self.azure_disk,
            azure_file: self.azure_file,
            cephfs: self.cephfs,
            cinder: self.cinder,
            config_map: self.config_map,
            csi: self.csi,
            downward_api: self.downward_api,
            empty_dir: self.empty_dir,
            ephemeral: self.ephemeral,
            fc: self.fc,
            flex_volume: self.flex_volume,
            flocker: self.flocker,
            gce_persistent_disk: self.gce_persistent_disk,
            git_repo: self.git_repo,
            glusterfs: self.glusterfs,
            host_path: self.host_path,
            iscsi: self.iscsi,
            name,
            nfs: self.nfs,
            persistent_volume_claim: self.persistent_volume_claim,
            photon_persistent_disk: self.photon_persistent_disk,
            portworx_volume: self.portworx_volume,
            projected: self.projected,
            quobyte: self.quobyte,
            rbd: self.rbd,
            scale_io: self.scale_io,
            secret: self.secret,
            storageos: self.storageos,
            vsphere_volume: self.vsphere_volume,
        }
    }
}

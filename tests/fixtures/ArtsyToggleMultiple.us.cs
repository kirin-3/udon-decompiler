// Lifted from the compiled Udon program; original comments and layout are unavailable.
using UdonSharp;
using UnityEngine;
using VRC.Udon.Common.Interfaces;

public class ArtsyToggleMultiple : UdonSharpBehaviour
{
    public global::UnityEngine.GameObject[] totoggle = new global::UnityEngine.GameObject[0];
    public global::UnityEngine.GameObject[] uwu = new global::UnityEngine.GameObject[0];
    public global::UnityEngine.GameObject[] owo = new global::UnityEngine.GameObject[0];
    public global::UnityEngine.AudioSource soundfx = null;
    private global::VRC.SDKBase.VRC_Pickup.PickupHand __const_VRCSDKBaseVRC_PickupPickupHand_0 =
        (global::VRC.SDKBase.VRC_Pickup.PickupHand)1;
    private global::VRC.SDKBase.VRC_Pickup.PickupHand __const_VRCSDKBaseVRC_PickupPickupHand_1 =
        (global::VRC.SDKBase.VRC_Pickup.PickupHand)2;

    public override void Interact()
    {
        global::System.Boolean __intnl_SystemBoolean_0 = false;
        global::System.Boolean __intnl_SystemBoolean_1 = false;
        global::System.Boolean __intnl_SystemBoolean_2 = false;
        global::UnityEngine.Object __intnl_UnityEngineObject_0 = null;
        global::UnityEngine.Object __intnl_UnityEngineObject_1 = null;
        global::System.Boolean __intnl_SystemBoolean_3 = false;
        global::VRC.SDKBase.VRCPlayerApi __intnl_VRCSDKBaseVRCPlayerApi_0 = null;
        global::VRC.SDKBase.VRCPlayerApi __intnl_VRCSDKBaseVRCPlayerApi_1 = null;
        global::System.Int32 i = 0;
        global::System.Int32 __intnl_SystemInt32_0 = 0;
        global::UnityEngine.GameObject __intnl_UnityEngineGameObject_0 = null;
        global::UnityEngine.GameObject __intnl_UnityEngineGameObject_1 = null;
        __intnl_SystemBoolean_0 = (global::System.Boolean)(totoggle != null);
        if (__intnl_SystemBoolean_0)
        {
            i = (global::System.Int32)(0);
            while (true)
            {
                __intnl_SystemInt32_0 = (global::System.Int32)(totoggle.Length);
                __intnl_SystemBoolean_1 = (global::System.Boolean)(__intnl_SystemInt32_0 > i);
                if (__intnl_SystemBoolean_1)
                {
                    __intnl_UnityEngineGameObject_0 =
                        (global::UnityEngine.GameObject)(totoggle[i]);
                    __intnl_UnityEngineGameObject_1 =
                        (global::UnityEngine.GameObject)(totoggle[i]);
                    __intnl_SystemBoolean_2 =
                        (global::System.Boolean)(__intnl_UnityEngineGameObject_1.activeSelf);
                    __intnl_SystemBoolean_3 =
                        (global::System.Boolean)(!__intnl_SystemBoolean_2);
                    __intnl_UnityEngineGameObject_0.SetActive(__intnl_SystemBoolean_3);
                    i = (global::System.Int32)(i + 1);
                    continue;
                }
                break;
            }
        }
        __intnl_SystemBoolean_1 = (global::System.Boolean)(uwu != null);
        if (__intnl_SystemBoolean_1)
        {
            i = (global::System.Int32)(0);
            while (true)
            {
                __intnl_SystemInt32_0 = (global::System.Int32)(uwu.Length);
                __intnl_SystemBoolean_2 = (global::System.Boolean)(__intnl_SystemInt32_0 > i);
                if (__intnl_SystemBoolean_2)
                {
                    __intnl_UnityEngineGameObject_0 = (global::UnityEngine.GameObject)(uwu[i]);
                    __intnl_UnityEngineGameObject_0.SetActive(true);
                    i = (global::System.Int32)(i + 1);
                    continue;
                }
                break;
            }
        }
        __intnl_SystemBoolean_2 = (global::System.Boolean)(owo != null);
        if (__intnl_SystemBoolean_2)
        {
            i = (global::System.Int32)(0);
            while (true)
            {
                __intnl_SystemInt32_0 = (global::System.Int32)(owo.Length);
                __intnl_SystemBoolean_3 = (global::System.Boolean)(__intnl_SystemInt32_0 > i);
                if (__intnl_SystemBoolean_3)
                {
                    __intnl_UnityEngineGameObject_0 = (global::UnityEngine.GameObject)(owo[i]);
                    __intnl_UnityEngineGameObject_0.SetActive(false);
                    i = (global::System.Int32)(i + 1);
                    continue;
                }
                break;
            }
        }
        __intnl_UnityEngineObject_0 = (global::UnityEngine.Object)(soundfx);
        __intnl_UnityEngineObject_1 = (global::UnityEngine.Object)(null);
        __intnl_SystemBoolean_3 = (global::System.Boolean)(__intnl_UnityEngineObject_0 !=
                                                           __intnl_UnityEngineObject_1);
        if (__intnl_SystemBoolean_3)
        {
            soundfx.Play();
        }
        __intnl_VRCSDKBaseVRCPlayerApi_0 =
            (global::VRC.SDKBase.VRCPlayerApi)(global::VRC.SDKBase.Networking.LocalPlayer);
        __intnl_VRCSDKBaseVRCPlayerApi_0.PlayHapticEventInHand(
            __const_VRCSDKBaseVRC_PickupPickupHand_0, 0.25f, 1.0f, 40.0f);
        __intnl_VRCSDKBaseVRCPlayerApi_1 =
            (global::VRC.SDKBase.VRCPlayerApi)(global::VRC.SDKBase.Networking.LocalPlayer);
        __intnl_VRCSDKBaseVRCPlayerApi_1.PlayHapticEventInHand(
            __const_VRCSDKBaseVRC_PickupPickupHand_1, 0.25f, 1.0f, 40.0f);
        return;
    }
}
<?php
# Generated by the protocol buffer compiler.  DO NOT EDIT!
# source: proto/v1/api.proto

use Google\Protobuf\Internal\GPBType;
use Google\Protobuf\Internal\RepeatedField;
use Google\Protobuf\Internal\GPBUtil;

/**
 * A function and module namespace that is defined outside of the current
 * module, and referenced & called by the current module.
 *
 * Generated from protobuf message <code>Import</code>
 */
class Import extends \Google\Protobuf\Internal\Message
{
    /**
     * Generated from protobuf field <code>string module_name = 1;</code>
     */
    protected $module_name = '';
    /**
     * Generated from protobuf field <code>.Function func = 2;</code>
     */
    protected $func = null;

    /**
     * Constructor.
     *
     * @param array $data {
     *     Optional. Data for populating the Message object.
     *
     *     @type string $module_name
     *     @type \PBFunction $func
     * }
     */
    public function __construct($data = NULL) {
        \GPBMetadata\Proto\V1\Api::initOnce();
        parent::__construct($data);
    }

    /**
     * Generated from protobuf field <code>string module_name = 1;</code>
     * @return string
     */
    public function getModuleName()
    {
        return $this->module_name;
    }

    /**
     * Generated from protobuf field <code>string module_name = 1;</code>
     * @param string $var
     * @return $this
     */
    public function setModuleName($var)
    {
        GPBUtil::checkString($var, True);
        $this->module_name = $var;

        return $this;
    }

    /**
     * Generated from protobuf field <code>.Function func = 2;</code>
     * @return \PBFunction|null
     */
    public function getFunc()
    {
        return isset($this->func) ? $this->func : null;
    }

    public function hasFunc()
    {
        return isset($this->func);
    }

    public function clearFunc()
    {
        unset($this->func);
    }

    /**
     * Generated from protobuf field <code>.Function func = 2;</code>
     * @param \PBFunction $var
     * @return $this
     */
    public function setFunc($var)
    {
        GPBUtil::checkMessage($var, \PBFunction::class);
        $this->func = $var;

        return $this;
    }

}

